use crate::kernels::*;
use crate::scene::{ Scene };
use crate::info::{ Info };
use crate::cl_helpers::{ create_five };
use crate::misc::{ load_source };

use ocl::{ Queue };

pub enum TraceProcessor{
    RealTracer(Box<TraceKernelReal>, Queue),
    AaTracer(Box<TraceKernelAa>, Box<ClearKernel>, Box<ImageKernel>, Queue)
}

impl TraceProcessor{
    pub fn new_real((width, height): (u32, u32), scene: &mut Scene, info: &mut Info) -> Result<Self, String>{
        info.start_time();
        let src = unpackdb!(load_source("assets/kernels/raytrace.clt"));
        info.set_time_point("Loading source file");
        let (_,_,_,program,queue) = unpackdb!(create_five(&src));
        info.set_time_point("Creating OpenCL objects");
        let kernel = unpackdb!(TraceKernelReal::new("raytracing", (width, height), &program, &queue, scene, info));
        info.set_time_point("Last time stamp");
        info.stop_time();
        info.print_info();
        Ok(TraceProcessor::RealTracer(Box::new(kernel), queue))
    }

    pub fn new_aa((width, height): (u32, u32), aa: u32, scene: &mut Scene, info: &mut Info) -> Result<Self, String>{
        info.start_time();
        let src = unpackdb!(load_source("assets/kernels/raytrace.cl"));
        info.set_time_point("Loading source file");
        let (_,_,_,program,queue) = unpackdb!(create_five(&src));
        info.set_time_point("Creating OpenCL objects");
        let kernel = unpackdb!(TraceKernelAa::new("raytracingAA", (width,height), aa, &program, &queue, scene, info));
        let clear_kernel = unpackdb!(ClearKernel::new("clear", (width,height), &program, &queue, kernel.get_buffer_rc()));
        let img_kernel = unpackdb!(ImageKernel::new("image_from_floatmap", (width,height), &program, &queue, kernel.get_buffer()));
        info.set_time_point("Last time stamp");
        info.stop_time();
        info.print_info();
        Ok(TraceProcessor::AaTracer(
            Box::new(kernel),
            Box::new(clear_kernel),
            Box::new(img_kernel),
            queue,
        ))
    }

    pub fn update(&mut self) -> Result<(), ocl::Error>{
        match self{
            TraceProcessor::RealTracer(kernel, queue) => kernel.update(queue),
            TraceProcessor::AaTracer(kernel, _, _, queue) => kernel.update(queue),
        }
    }

    pub fn render(&mut self) -> Result<&[i32], ocl::Error>{
        match self{
            TraceProcessor::RealTracer(kernel, queue) =>{
                kernel.execute(queue)?;
                kernel.get_result(queue)
            },
            TraceProcessor::AaTracer(kernel, clear_kernel, img_kernel, queue) =>{
                clear_kernel.execute(queue)?;
                kernel.execute(queue)?;
                img_kernel.execute(queue)?;
                img_kernel.get_result(queue)
            },
        }
    }
}
