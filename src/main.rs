extern crate byteorder;
extern crate tempfile;
#[macro_use]
extern crate wayland_client;
extern crate cairo;
extern crate glib;
extern crate gdk_pixbuf;

use byteorder::{WriteBytesExt, NativeEndian, LittleEndian, BigEndian};

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::thread::sleep;
use std::time::Duration;

use wayland_client::wayland::get_display;
use wayland_client::wayland::compositor::WlCompositor;
use wayland_client::wayland::shell::{WlShell, WlShellSurfaceFullscreenMethod};
use wayland_client::wayland::shm::{WlShm, WlShmFormat};

use cairo::{ImageSurface, Format, Context};
use glib::translate::ToGlibPtr;
use gdk_pixbuf::Pixbuf;

wayland_env!(WaylandEnv,
    compositor: WlCompositor,
    shell: WlShell,
    shm: WlShm
);

fn read_from_file(file: &str) -> Pixbuf {
    Pixbuf::new_from_file(file)
        .expect("Couldn't read file")
}

fn main() {
    let (display, iter) = match get_display() {
        Ok(d) => d,
        Err(e) => panic!("Unable to connect to a wayland compositor: {:?}", e)
    };

    // Use wayland_env! macro to get the globals and an event iterator
    let (env, mut evt_iter) = WaylandEnv::init(display, iter);

    // Get shortcuts to the globals.
    // Here we only use the version 1 of the interface, so no checks are needed.
    let compositor = env.compositor.as_ref().map(|o| &o.0).unwrap();
    let shell = env.shell.as_ref().map(|o| &o.0).unwrap();
    let shm = env.shm.as_ref().map(|o| &o.0).unwrap();

    let surface = compositor.create_surface();
    let shell_surface = shell.get_shell_surface(&surface);

    // create a tempfile to write on
    use std::env::args;
    let mut args = args();
    args.next();
    let file_path = &*args.next()
        .unwrap_or("/home/timidger/Pictures/2016-04-21-220859_984x560_scrot.png".into());
    let pixbuf = read_from_file(file_path);
    let mut tmp = tempfile::tempfile().ok().expect("Unable to create a tempfile.");
    let pixels = unsafe { pixbuf.get_pixels()};
    let width = pixbuf.get_width();
    let height = pixbuf.get_height();
    let stride = pixbuf.get_rowstride();
    let colorspace = pixbuf.get_colorspace();
    let channels = pixbuf.get_n_channels();
    let length = pixels.len();
    println!("{}x{} w/ stride {}, bits/sample: {}, colorspace: {}, channels: {}",
             width, height, stride, pixbuf.get_bits_per_sample(), colorspace, channels);
    for (index,chunk) in pixels.chunks(3).enumerate() {
        let mut chunked_pixels = [0u8; 4];
        if (index as i32 % stride) > width {
            println!("Never");
            chunked_pixels[0] = 255;
            chunked_pixels[1] = 255;
            chunked_pixels[2] = 255;
            chunked_pixels[3] = 255;
            /*for _ in 0..(stride-width) / 4{
                unsafe {tmp.write_u32::<BigEndian>(::std::mem::transmute(chunked_pixels))};
            }*/
            continue;
        }
        chunked_pixels[0] = 255;
        for (&x, p) in chunk.iter().zip(chunked_pixels[1..].iter_mut()) {
            *p = x;
        }
        unsafe {tmp.write_u32::<BigEndian>(::std::mem::transmute(chunked_pixels))};
    }
    // stupid raw pointer thing
    //for _ in 0..height {
    //}
    //tmp.write_all(pixels).expect("Couldn't write background");

    /*for chunk in pixels.chunks(3) {
        let mut chunked_pixels = [0u8; 4];
        chunked_pixels[0] = 255;
        for (&x, p) in chunk.iter().zip(chunked_pixels[1..].iter_mut()) {
            *p = x;
        }
        unsafe {tmp.write_u32::<BigEndian>(::std::mem::transmute(chunked_pixels))};
    }*/
    //tmp.write_all(pixels).expect("Couldn't write background");


    tmp.flush()
        .expect("Couldn't flush background to temp file");

    let pool = shm.create_pool(tmp.as_raw_fd(), (height * stride) as i32);
    let buffer = pool.create_buffer(0, width, height, stride, WlShmFormat::Argb8888);

    // make our surface as a toplevel one
    shell_surface.set_toplevel();
    //shell_surface.set_fullscreen(WlShellSurfaceFullscreenMethod::Scale, 0, None);
    shell_surface.set_class("Background".into());
    // attach the buffer to it
    surface.attach(Some(&buffer), 0, 0);
    surface.set_buffer_scale(4);
    // commit
    surface.commit();

    evt_iter.sync_roundtrip().unwrap();

    loop {sleep(Duration::new(1,0));}

}
