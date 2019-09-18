
//extern crate dinotreedemo;
extern crate dinotreedemomenu;
extern crate axgeom;
use dinotreedemomenu::*;
use axgeom::*;
use std::os::raw::{c_char, c_int};
use std::ffi::CStr;
use std::ptr::null_mut;



#[repr(transparent)]
#[derive(Copy,Clone,Debug,Default)]
pub struct Vertex(pub [f32;2]);



#[repr(C)]
struct Repr<T>{
    ptr:*const T,
    size:usize,
}

#[repr(C)]
struct ReprMut<T>{
    ptr:*mut T,
    size:usize,
}

fn rust_game_create(startx:usize,starty:usize,radius:&mut f32,border:&mut [f32;4],icolor:&mut [f32;3])->(*mut MenuGame){
    rayon::ThreadPoolBuilder::new().num_threads(num_cpus::get_physical()).build_global().unwrap();

    
    //let border=unsafe{&mut (*border)};
	let (mm,game_response)=MenuGame::new();

    let diff=game_response.new_game_world.unwrap();
	
    let mut bor=dinotreedemomenu::compute_border(diff.0,[startx as f32,starty as f32]);
        
    let ((a,b),(c,d))=bor.get();

    border[0]=a;
    border[1]=b;
    border[2]=c;
    border[3]=d;

    *radius=diff.1;

    let cols=&game_response.color.unwrap();
    icolor[0]=cols[0];
    icolor[1]=cols[1];
    icolor[2]=cols[2];



    Box::into_raw(Box::new(mm))
}

fn rust_game_destroy(menu:&mut MenuGame){
	 drop(unsafe{Box::from_raw(menu as *mut MenuGame)});
}

fn rust_game_step(
        menu:&mut MenuGame,
        startx:usize,
        starty:usize,
        poses:&[f32],
        border:&[f32],
        //output
        icolor:&mut [f32],
        new_border:&mut [f32],
        radius:&mut f32,
        num_verticies:&mut i32)->bool{
	/* //TODO do this after
    if menu.is_null(){
		return false;
	}
	*/

    //let menu=unsafe{&mut (*menu)};


    let poses:&[Vec2<f32>]={
        assert_eq!(poses.len()%2,0);
        let decompose:Repr<f32>=unsafe{std::mem::transmute(poses)};

        unsafe{std::mem::transmute(Repr{ptr:decompose.ptr,size:decompose.size/2})}
	};

    let border:Rect<f32>={
        assert_eq!(border.len(),4);
        //TODO just transmute instead?
        Rect::new(border[0],border[1],border[2],border[3])
    };

    let icolor:&mut [f32]={
        icolor
    };

    let new_border:&mut Rect<f32>={
        assert_eq!(new_border.len(),4);
        let decompose:ReprMut<f32>=unsafe{std::mem::transmute(new_border)};
        unsafe{std::mem::transmute(decompose.ptr)}
    };

    let radius:&mut f32={
        radius
    };

    let num_verticies:&mut i32={
        num_verticies
    };


    let game_response = menu.step(poses,&border);

    match game_response.new_game_world{
        Some((bod,rad))=>{
            *radius=rad;
            *new_border=dinotreedemomenu::compute_border(bod,[startx as f32,starty as f32]);
        },
        None=>{}
    }

    match game_response.color{
        Some(col)=>{
            icolor.clone_from_slice(&col);
        },
        None=>{}
    }

    *num_verticies=menu.get_bots().len() as i32;


    let is_game=game_response.is_game;


	return is_game;
}


fn rust_game_update_verticies(menu:*mut MenuGame,verts:&mut [f32]){
    let menu=unsafe{&mut (*menu)};

    //Why does htis break it???
    // assert_eq!(2*verts.len(),menu.get_bots().len(),"sizes do not match!!!");
    let verts:&mut [Vertex]={
        //assert_eq!(verts.len() % 2,0);
            
        let decompose:ReprMut<f32>=unsafe{std::mem::transmute(verts)};

        unsafe{std::mem::transmute(ReprMut{ptr:decompose.ptr,size:decompose.size/2})}
    };

    for (a,b) in menu.get_bots().iter().zip(verts.iter_mut()){
        *b=Vertex([a.pos.x,a.pos.y]);
    }
}






/// Expose the JNI interface for android below
//#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;
    use std;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString};
    use self::jni::sys::{jint, jlong};
    use self::jni::sys::jfloatArray;
    use self::jni::sys::jintArray;
    use self::jni::sys::jboolean;
    use self::jni::objects::JByteBuffer;
    use self::jni::sys::jobject;

    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameCreate(env: JNIEnv, _: JClass, startx:jint,starty:jint,radius: jfloatArray, border: jfloatArray,colors:jfloatArray) -> jlong {
        

        let mut mradius:[f32;1]=[0.0];
        let mut mborder:[f32;4]=[0.0;4];
        let mut mcolor:[f32;3]=[0.0;3];

        let k=rust_game_create(startx as usize,starty as usize,&mut mradius[0],&mut mborder,&mut mcolor);
        


        env.set_float_array_region(colors,0,&mcolor);
        env.set_float_array_region(border,0,&mborder);
        env.set_float_array_region(radius,0,&mradius);
                

        if(conv::into_pointer(conv::into_jlong(k))!=k){
            return 0;
        }else{
            conv::into_jlong(k)
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameDestroy(env: JNIEnv, _: JClass, game: jlong){
        let game=conv::into_pointer(game);
        rust_game_destroy(unsafe{&mut *game});
    }


    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_updateVerticies(env: JNIEnv, _: JClass,game:jlong,jverts:JByteBuffer){
        let game= unsafe{&mut*conv::into_pointer(game)};


        let verts:&mut [f32]={       
            let (buffptr,bufflen)={
                let buff:&mut [u8]=env.get_direct_buffer_address(jverts).unwrap();
                let k:ReprMut<u8>=unsafe{std::mem::transmute(buff)};
                (k.ptr,k.size)
            };

            let new_len=(bufflen/4);
            //let kk:&mut [f32;2]=unsafe{std::mem::transmute(buffptr)};
            unsafe{std::mem::transmute(Repr{ptr:buffptr,size:new_len})}
        };


        rust_game_update_verticies(game,verts)
    }

    
    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameStep(
            env: JNIEnv,
             _: JClass,
            game:jlong,
            startx:jint,
            starty:jint,
            poses:jfloatArray,
            border:jfloatArray,
            //outputs
            new_border:jfloatArray,
            radius:jfloatArray,
            colors:jfloatArray,
            num_verticies:jintArray)->jint  {
        

        let poses={
            let len=env.get_array_length(poses).unwrap();
            let mut po=(0..len).map(|_|0.0).collect::<Vec<f32>>();    
            env.get_float_array_region(poses,0,&mut po).unwrap();
            po
		};

        let current_border={
            let len=env.get_array_length(border).unwrap();
            let mut bo=(0..len).map(|_|0.0).collect::<Vec<f32>>();    
            env.get_float_array_region(border,0,&mut bo).unwrap();
            bo
        };


        let mut mradius:[f32;1]=[0.0];
        let mut mborder:[f32;4]=[0.0;4];
        let mut mcolor:[f32;3]=[0.0;3];
        let mut mnum_verticies:[i32;1]=[0];


        let game=conv::into_pointer(game);

        
		let is_game = rust_game_step(unsafe{&mut *game},startx as usize,starty as usize,&poses,&current_border,&mut mcolor,&mut mborder,&mut mradius[0],&mut mnum_verticies[0]);



        env.set_float_array_region(colors,0,&mcolor);
        env.set_float_array_region(new_border,0,&mborder);
        env.set_float_array_region(radius,0,&mradius);
        env.set_int_array_region(num_verticies,0,&mnum_verticies);
                
        if is_game{
            1
        }else{
            0
        }
    }
    




    #[cfg(target_pointer_width = "32")]
    mod conv{
        use super::*;
        pub fn into_jlong(a:*mut MenuGame)->jlong{
            let a:u32=unsafe{std::mem::transmute(a)};

            a as jlong
        }
        pub fn into_pointer(a:jlong)->*mut MenuGame{
            //let mut a:u32=(a.to_be()&0x00000000FFFFFFFF) as u32;
            //u32::from_be(a);
            let a=a as u32;
            unsafe{std::mem::transmute(a)}
        }
    }
    #[cfg(target_pointer_width = "64")]
    mod conv{
        use super::*;
        pub fn into_jlong(a:*mut MenuGame)->jlong{
            unsafe{std::mem::transmute(a)}
        }
        pub fn into_pointer(a:jlong)->*mut MenuGame{
            unsafe{std::mem::transmute(a)}
        }
    }
    

}


