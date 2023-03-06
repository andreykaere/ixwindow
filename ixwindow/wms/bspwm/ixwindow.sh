#!/bin/bash

PREFIX="$1"
# Folder for icons
CACHE_DIR="$2"
GAP="$3"
X="$4"
Y="$5"
SIZE="$6"
COLOR="$7"

PREV_ICON="/tmp/polybar-icon-prev"

# Ideally, this function is supposed to remove just previous icon. However, 
# practice has shown, that it is not a very good solution in real life (see 
# the note below). In any case, this function is needed, becase if there will 
# be many (overlaping) icons, it may cause a slow down of your window manager.
#
# Note: if user switches windows fast enough, program might not remove some
# icons, so it was chosen to search for all possibly not deleted icons every 
# time we switch windows. This way, if some icons were missed, they will be 
# removed next time, when user will not switch windows this fast.
remove_all_prev_icons() {
    local icons_ids=($(xdo id -n "polybar-ixwindow-icon" 2> /dev/null))
    for icon in "${icons_ids[@]}";
    do
        xdo kill "$icon" &> /dev/null 
    done
}



print_info() {
    echo -n "$GAP"

    if [ "$1" = "Empty" ]; then
        echo "Empty"
    else
        local wid="$1"
        
        # Doesn't always work, so xprop is more realiable here 
        # WM_CLASS="$(bspc query -T -n "$Node" | jq -r '.client.className')"  
        local WM_CLASS="$(get_wm_class "$wid")"
        
        case "$WM_CLASS" in
            'Brave-browser')
                echo "Brave"
                ;;
            'TelegramDesktop')
                echo "Telegram"
                ;;
            *)
                # https://stackoverflow.com/questions/1538676/uppercasing-first-letter-of-words-using-sed
                echo "$WM_CLASS" | sed -e "s/\b\(.\)/\u\1/g"
                ;;
        esac
    fi
}


exists_fullscreen_node() {
    local fullscreen_nodes="$(bspc query -N -n .fullscreen.\!hidden -d "$1")"

    if [ -n "$fullscreen_nodes" ]; then 
        echo '1'
    else
        echo '0'
    fi
}


display_icon() {
    remove_all_prev_icons
    icon="$CACHE_DIR/$1.jpg"

    if [ -f "$icon" ]; then
        "$PREFIX/bspwm/polybar-ixwindow-icon" "$icon" "$X" "$Y" "$SIZE" &> /dev/null &
    fi
}


generate_icon() {
    # Needed for some applications, because sometimes icon is not added right
    # away and some pause is needed
    sleep 0.5
    "$PREFIX/generate-icon" "$CACHE_DIR" "$SIZE" "$COLOR" "$1"
}


reset_prev_icon() {
    echo "" > "$PREV_ICON"
}


update_prev_icon() {
   echo "$1" > "$PREV_ICON" 
}

get_wm_class() {
    echo "$(xprop -id "$1" WM_CLASS | awk -F '=' '{print $2}' | \
    awk -F ',' '{print $2}' |  tr -d '"' | \
    sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
}

process_window() {
    local desk="$2"    
    local wid="$1"
    local WM_CLASS="$(get_wm_class "$wid")"
    
    # If there is a fullscreen node, don't show anything, 
    # since we shouldn't see it
    if [ "$(exists_fullscreen_node "$desk")" = "1" ]; then
        remove_all_prev_icons
        reset_prev_icon
        return 0;
    fi

    generate_icon "$wid"
   
    # We use icon-prev thing just so icon won't blink 
    # when one is switching between the same types of windows
    local WM_CLASS_PREV="$(cat "$PREV_ICON")"

    if [ "$WM_CLASS" = "$WM_CLASS_PREV" ]; then
        return 0;
    else
        update_prev_icon "$WM_CLASS"
    fi
   

    display_icon "$WM_CLASS"

    print_info "$wid"
}


is_desktop_empty() {
    local desk="$1"
    local nodes="$(bspc query -N -n .window.\!hidden  -d "$desk")"


    if [ -n "$nodes" ]; then 
        echo '0'
    else
        echo '1'
    fi
}

process_desktop() {
    local desk="$1"
    local is_empty="$(is_desktop_empty "$desk")"

    if [ "$is_empty" = "1" ]; then
        reset_prev_icon
        remove_all_prev_icons 

        print_info "Empty"
    fi

}


reset_prev_icon


bspc subscribe node_focus | while read -r Event Monitor Desktop Node
do
    # For some reason "$Node" and "$Desktop" are not always working 
    # properly with sticky windows
    Node="$(xdotool getactivewindow)"
    Desktop="$(bspc query -D -d focused)"

    process_window "$Node" "$Desktop"

done &


bspc subscribe node_state | while read -r Event Monitor Desktop Node State Active 
do
    Node="$(xdotool getactivewindow)"
    Desktop="$(bspc query -D -d focused)"

    if [ "$State" != "fullscreen" ]; then
        continue;
    fi
    
    # So, if you will focus on the other windows of the same app,
    # which are not fullscreen, you will see icon

    reset_prev_icon

    if [ "$Active" = "on" ]; then
        remove_all_prev_icons
    else
        process_window "$Node"  "$Desktop"
    fi

done &


bspc subscribe node_add | while read -r Event Monitor Desktop Ip Node 
do
    State="$(bspc query -T -n "$Node" | jq -r '.client.state')"

    if [ "$State" = "fullscreen" ]; then
        reset_prev_icon
        remove_all_prev_icons
    fi

done &


bspc subscribe node_flag | while read -r Event Monitor Desktop Node Flag Active 
do
    if [ "$Flag" = "hidden" ] && [ "$Desktop" = "$(bspc query -D -d .focused)" ]; then
        process_desktop "$Desktop"
    fi

done &




bspc subscribe node_remove | while read -r Event Monitor Desktop Node 
do
    process_desktop "$Desktop"

done &


bspc subscribe desktop_focus | while read -r Event Monitor Desktop 
do

    if [ "$(exists_fullscreen_node "$Desktop")" = "1" ]; then
        reset_prev_icon
        remove_all_prev_icons
        continue;
    fi

    process_desktop "$Desktop"

done 
