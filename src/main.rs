
//this program should run an nbody sim on many spheres, using legion as ecs system and nphysics for
//physics

//**DONE */





/****************************************** */

//**TODO!!!*/
//render pipeline
//*run systems in main event loop
//*iterate (parallel) over entites and components in event loop
//*apply physics to entity with nphysics
//*render sphere textures? idk i want BALLS!
//*usable camerea / world system (zoomout, arrow key to pan, shift clk to pan)
//*usable coordinate system
//*spawn and select bodys with mouse, drag around bodies
//*add settings menu...
//*UI init
//*Profiler available


//**WISHLIST */
//*use emuGPU to do physics and collision calculations instead of CPU.
//*use emuGPU to do nbody calculations
//*if needed, floating origin solution
//*add different sphere types
//*add temperature simulation
//*add chemical simulation (spheres bonding together to Globs)



/*ecs based gpu accelerated physics is the move.*/
// is it tho
//what is the move
//keep it simple
//no UI
//no text
//all discovery
//intuition
//maf
//loloololol

//use core::num::dec2flt::rawfp::encode_subnormal;



mod rendition;
use std::time::Instant;

use futures::{FutureExt, executor::block_on};
use winit::{dpi::Size, event::*, event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}};


fn main(){


    let size = winit::dpi::LogicalSize::new(84,48);

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    //since main cant be async, we need a block
    let mut state = block_on(rendition::State::new(&window));



    let mut updatecount = 0u16;
    let mut last_now: Option<Instant> = Some(Instant::now());

    event_loop.run(move |event, _, control_flow| 
    {


        
        match event 
        {
            Event::WindowEvent 
            {
                ref event,
                window_id,
            } 
            if window_id == window.id() => if !state.input(event) 
            { 
                match event 
                {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput 
                    {
                        input,
                    ..
                    } => 
                    {
                        match input 
                        {
                            KeyboardInput 
                            {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                        _ => {}
                        }
                    }
                    WindowEvent::Resized(physical_size) =>
                    {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged{new_inner_size, ..} =>
                    {
                        //new_inner_size is &&mut so must be dereferenced twice to get actual value
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(_) =>
            {
                //update before each redraw, but after input is handled
                state.update();
                match state.render()
                {
                    Ok(_) => 
                    {
                        //for fps timing
                        let now = Instant::now();
                        update_fps(&mut updatecount,&mut last_now,now);
                    }
                    //recreate the swapchain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    //system out of memory, so exit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    //all other errors should be resolved by the next frame lul
                    Err(e) => eprintln!("{:?}",e),
                }
            }
            Event::MainEventsCleared => {
                //RedrawRequested will run once, must be requested.
                window.request_redraw();

            }

            _ => {}
        }


    });
}

fn update_fps(updatecount: &mut u16,last_now:&mut Option<Instant>, now:Instant){

            //update fps once a sec
            let update_text_string = match last_now 
            {
                Some(last_update_instant) => 
                {
                    *updatecount = *updatecount+1;
                    (now - *last_update_instant).as_secs_f32() >= 1.0
                },
                None => true,
            };
                    
            //update fps text
            if update_text_string 
            {
                let fps = *updatecount;
                let fps_text = format!("Fps: {:.1}", fps);
                *last_now = Some(now);
                *updatecount = 0;
                println!("{}",fps_text);
            }
            


}