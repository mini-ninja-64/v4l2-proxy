use std::io::{self, Error, ErrorKind};
use std::{io::stdin, path::PathBuf};

use clap::Parser;
use libcamera::camera_manager::{self, CameraManager};
use libcamera::framebuffer_allocator::FrameBufferAllocator;
use libcamera::framebuffer_map::MemoryMappedFrameBuffer;
use libcamera::geometry::Size;
use libcamera::pixel_format::PixelFormat;
use libcamera::stream::StreamRole;
use usb_gadget::function::custom::{Custom, Endpoint, EndpointDirection, Interface};
use v4l::capability::Flags;
use v4l::video::{Capture, Output};
use usb_gadget::{default_udc, Gadget, Class, Id, Strings, Config};
use usb_gadget::function::uvc::{Uvc, Frame, UvcBuilder};
use v4l::FourCC;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    camera: Option<usize>
}

fn wait() {
    let mut buff = String::new();
    let _ = stdin().read_line(&mut buff);
}

const MJPEG_FOURCC: FourCC = FourCC { repr: *b"MJPG" };
const YUYV_FOURCC: FourCC = FourCC { repr: *b"YUYV" };
const PIXEL_FORMAT_MJPEG: PixelFormat = PixelFormat::new(u32::from_le_bytes(*b"MJPG"), 0);

fn main() -> io::Result<()> {
    let args = Args::parse();

    let udc_device = default_udc()?;
    let mut builder = Uvc::builder();
    builder.add_frame(&Frame {
        format: "mjpeg",
        name: "mjpeg",
        width: 1920,
        height: 1080,
        frame_intervals: vec![UvcBuilder::fps(30)]
    });
    let (uvc, handle) = builder.build();
    let gadget = Gadget::new(
        Class::new(255, 255, 3),
        Id::new(6, 0x11),
        Strings::new("mini", "Rust UVC proxy", "serial_number"),
    )
    .with_config(Config::new("config")
    .with_function(handle))
    .bind(&udc_device)?;

    println!("handle: {:?}", gadget.path());

    // let sink_v4l = uvc.get_v4l_device()
    //     .and_then(|o| v4l::Device::with_path(o));
    // if let Ok(sink_v4l) = sink_v4l {
    //     // println!("Sink formats:\n{:#?}", Output::enum_formats(&sink_v4l)?);
    //     println!("Sink frame sizes:\n{:#?}", Output::enum_framesizes(&sink_v4l, MJPEG_FOURCC)?);
    //     println!("Sink capabilities:\n{}", sink_v4l.query_caps()?);
    //     println!("Sink format:\n{}", Output::format(&sink_v4l)?);
    // }

    wait();

    gadget.remove()?;
    // https://github.com/lit-robotics/libcamera-rs/issues/2#issuecomment-1430644226
    // https://github.com/raymanfx/libv4l-rs/blob/6d171419c18659b54a6570a1294c047a8d704548/examples/stream_forward_mmap.rs#L9
    Ok(())
}
