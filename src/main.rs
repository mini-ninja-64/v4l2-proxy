use std::{io::stdin, path::PathBuf};

use clap::Parser;
use usb_gadget::function::custom::{Custom, Endpoint, EndpointDirection, Interface};
use v4l::capability::Flags;
use v4l::video::Capture;
use usb_gadget::{default_udc, Gadget, Class, Id, Strings, Config};
use usb_gadget::function::uvc::{Uvc, Frame, UvcBuilder};
use v4l::FourCC;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    v4l: Option<PathBuf>
}

fn wait() {
    let mut buff = String::new();
    let _ = stdin().read_line(&mut buff);
}

const MJPEG_FOURCC: FourCC = FourCC { repr: [77,74,80,71] };
const YUYV_FOURCC: FourCC = FourCC { repr: [89,85,89,86] };

fn main() {
    let args = Args::parse();

    let inbound_v4l = args.v4l
        .map(|path| v4l::Device::with_path(path))
        .unwrap_or_else(|| v4l::Device::new(0))
        .expect("v4l device could not be loaded");

    let supports_streaming = inbound_v4l
    .query_caps()
    .map(|c| c.capabilities.contains(Flags::STREAMING))
    .unwrap_or(false);
    if !supports_streaming {
        panic!("Input v4l device does not support streaming, gonna esplode now :3");
    }

    println!("v4l_device: {:#?}", inbound_v4l
        .enum_formats()
        .map(|formats| formats.iter()
            .map(|format| format.fourcc)
            .collect::<Vec<_>>()
        ));
    
    let udc_device = default_udc().expect("udc could not be loaded");
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
    .bind(&udc_device)
    .expect("UVC gadget could not be registered with UDC");

    let outbound_v4l = uvc.get_v4l_device()
        .and_then(|o| v4l::Device::with_path(o));
    if outbound_v4l.is_ok() {
        let outbound_v4l = outbound_v4l.unwrap();
    }

    wait();

    gadget.remove().expect("Unable to unregister UVC gadget");
}
