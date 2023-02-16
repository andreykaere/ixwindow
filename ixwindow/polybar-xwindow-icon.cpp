// compilation:
// g++ polybar-xwindow-icon.cpp -o polybar-xwindow-icon -I/usr/include/opencv4/ -lopencv_core -lopencv_videoio -lopencv_highgui -lopencv_imgcodecs -lopencv_imgproc -lX11
//
// Code is mostly copied from 
// https://stackoverflow.com/questions/54513419/putting-image-into-a-window-in-x11

// Some parts are taken from
// https://stackoverflow.com/questions/57078155/draw-border-frame-using-xlib



#include <opencv2/opencv.hpp>   // FOR OpenCV
#include <opencv2/core/core.hpp>     // Basic OpenCV structures (cv::Mat)
#include <opencv2/video/video.hpp>  
#include <opencv2/imgproc/imgproc.hpp>
#include <opencv2/highgui/highgui.hpp>

#include <bits/stdc++.h>
#include <X11/Xlib.h> // Every Xlib program must include this
#include <assert.h>   // I include this to test return values the lazy way
#include <unistd.h>   // So we got the profile for 10 seconds
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include <X11/Xlib.h> // Every Xlib program must include this
#include <X11/Xatom.h>
#include <X11/extensions/Xcomposite.h>
#include <X11/extensions/Xfixes.h>
#include <X11/extensions/shape.h>
#define NIL (0)       // A name for the void pointer

using namespace cv;
using namespace std;

int main(int argc, char** argv) {
    
    if (argc < 5) {
        cout << "Not enough arguments";
        return -1;
    }
    
    int x, y, size;
    char* filename;

    filename = argv[1];
    sscanf(argv[2], "%d", &x);
    sscanf(argv[3], "%d", &y);
    sscanf(argv[4], "%d", &size);

    XGCValues gr_values;
    //GC gc;
    XColor    color, dummy;


    Display *dpy = XOpenDisplay(NIL);
    //assert(dpy);
    //int screen = DefaultScreen(dpy);
    // Get some colors

    int blackColor = BlackPixel(dpy, DefaultScreen(dpy));
    int whiteColor = WhitePixel(dpy, DefaultScreen(dpy));

    

    // Create the window
    Window w = XCreateSimpleWindow(dpy, DefaultRootWindow(dpy), x, y, 
                 size, size, 0, whiteColor, blackColor);
    
    // We don't want window manager to handle icon window
    XSetWindowAttributes attributes;
    attributes.override_redirect = true;

    XChangeWindowAttributes(dpy, w, CWOverrideRedirect, &attributes);

    // We want to get MapNotify events

    XSelectInput(dpy, w, StructureNotifyMask);
    // XSelectInput(dpy, w, ExposureMask);

    long value = XInternAtom(dpy, "_NET_WM_WINDOW_TYPE_DOCK", False);

    XChangeProperty(dpy, w, XInternAtom(dpy, "_NET_WM_WINDOW_TYPE", False),
                   XA_ATOM, 32, PropModeReplace, (unsigned char *) &value, 1);

    XClassHint *polybar_xwindow_icon;
    
    //my_struct = malloc(sizeof(t_data));
    polybar_xwindow_icon = XAllocClassHint();
    polybar_xwindow_icon->res_name = (char*) "polybar-xwindow-icon";
    polybar_xwindow_icon->res_class = (char*) "Polybar-xwindow-icon";

    XSetClassHint(dpy, w, polybar_xwindow_icon);
    XFree(polybar_xwindow_icon);


    XMapWindow(dpy, w);

    // Wait for the MapNotify event

    for(;;) {
        XEvent e;
        XNextEvent(dpy, &e);
        if (e.type == MapNotify)
            break;
    }

    Window focal = w;

    XWindowAttributes gwa;
    XGetWindowAttributes(dpy, w, &gwa); 
    int wd1 = gwa.width;
    int ht1 = gwa.height;



    XImage *image = XGetImage(dpy, w, 0, 0 , wd1, ht1, AllPlanes, ZPixmap);
    unsigned long rm = image->red_mask;
    unsigned long gm = image->green_mask;
    unsigned long bm = image->blue_mask;

    Mat img(ht1, wd1, CV_8UC3);     // OpenCV Mat object is initilaized
    Mat scrap = imread(filename);//(wid, ht, CV_8UC3);      
    resize(scrap, img, img.size(), cv::INTER_AREA);

    for(int x = 0; x < wd1; x++) {
        for(int y = 0; y < ht1; y++) {
            unsigned long pixel = XGetPixel(image, x, y);     

            Vec3b color = img.at<Vec3b>(Point(x, y));



            pixel = 65536 * color[2] + 256 * color[1] + color[0];               

            XPutPixel(image, x, y, pixel);                  
        }
    }

    // namedWindow("QR", CV_WINDOW_NORMAL);
    // imshow("QR", img);


    GC gc = XCreateGC(dpy, w, 0, NIL);
    XPutImage(dpy, w, gc, image, 0, 0, 0, 0, wd1, ht1);

    int run = 1;

    while(run) {
        XEvent xe;
        XNextEvent(dpy, &xe);
        switch (xe.type) {
            case Expose:
                XPutImage(dpy, w, gc, image, 0, 0, 0, 0, wd1, ht1);
                XSetForeground(dpy, gc, color.pixel);
                XSync(dpy, False);
                break;

            default:
                break;
        }
    }

    XDestroyWindow(dpy, w);
    XCloseDisplay(dpy);


    // waitKey(0);    
    return 0;
}
