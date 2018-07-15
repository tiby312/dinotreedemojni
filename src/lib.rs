
extern crate dinotreedemo;
extern crate axgeom;
extern crate rayon;

use dinotreedemo::MenuGame;

use std::os::raw::{c_char, c_int};
use std::ffi::CStr;
use std::ptr::null_mut;
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

fn rust_game_create(startx:c_int,starty:c_int)->(*mut MenuGame){
	let mm=MenuGame::new(startx as usize,starty as usize);
	Box::into_raw(Box::new(mm.0))
}

fn rust_game_destroy(menu:*mut MenuGame){
	 drop(unsafe{Box::from_raw(menu)});
}

fn rust_game_step(menu:*mut MenuGame,poses:*const [f32;2],num_poses:usize,icolor:*mut f32,verts:*mut [f32;2],size:usize)->bool{
	
    if menu.is_null(){
		return false;
	}
	

    let menu=unsafe{&mut (*menu)};

    let verts={
            #[repr(C)]
        struct Repr<T>{
            ptr:*const T,
            size:usize,
        }

        let k:&mut [dinotreedemo::Vert]=unsafe{std::mem::transmute(Repr{ptr:verts,size:size})};
        k 
    };

	let tr=Repr{ptr:poses,size:num_poses};

	let poses:&[dinotreedemo::vec::Vec2]=unsafe{std::mem::transmute(tr)};
	let (color,is_game) = menu.step(poses,verts);

    match color{
        Some(color)=>{
            let jj:&mut [f32]=unsafe{std::mem::transmute(Repr{ptr:icolor,size:3})};
            jj.clone_from_slice(&color);
            //jj[0]=rayon::current_num_threads() as f32;
            
        },
        None=>{}
    }


    //use rayon;
    
    //println!("num threads={:?}",rayon::current_num_threads());


	return is_game;
}


fn rust_game_num_verticies(menu:*mut MenuGame)->usize{
    let menu=unsafe{&mut (*menu)};

    menu.get_num_verticies()
}
/*
fn rust_game_verticies(menu:*mut MenuGame,verts:*mut [f32;2],size:usize){
	let menu=unsafe{&mut (*menu)};

	#[repr(C)]
	struct Repr<T>{
	    ptr:*const T,
	    size:usize,
	}

    let k:&mut [dinotreedemo::Vert]=unsafe{std::mem::transmute(Repr{ptr:verts,size:size})};
    menu.get_verticies(k);
	//let k:Repr<[f32;2]>=unsafe{std::mem::transmute(menu.get_verticies())};
	//(k.ptr,k.size)
}
*/

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
    use self::jni::objects::JByteBuffer;
    use self::jni::sys::jobject;

    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameCreate(env: JNIEnv, _: JClass, startx: jlong, starty: jlong) -> jlong {
        

        let k=rust_game_create(startx as c_int,starty as c_int);
        
        if(conv::into_pointer(conv::into_jlong(k))!=k){
            return 0;
        }else{
            conv::into_jlong(k)
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameDestroy(env: JNIEnv, _: JClass, game: jlong){
        //let game:*mut MenuGame=std::mem::transmute(game);
        let game=conv::into_pointer(game);
        rust_game_destroy(game);
    }


    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameNumVerticies(env: JNIEnv, _: JClass,game:jlong)->jint{
        
        let game= unsafe{&mut*conv::into_pointer(game)};
        rust_game_num_verticies(game) as jint
        //game.get_num_verticies().len() as jint
    }
    #[no_mangle]
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameStep(env: JNIEnv, _: JClass, game:jlong,poses:jfloatArray,colors:jfloatArray,jverts:JByteBuffer) ->jint {
        

        let len=env.get_array_length(poses).unwrap();
        //let mut buf:Vec<f32>=Vec::with_capacity(len as usize);
        let mut buf=(0..len).map(|_|0.0).collect::<Vec<f32>>();
        

        let (poses_ptr,poses_size)={
            let buf2:&mut [f32]=&mut buf;
            env.get_float_array_region(poses,0,buf2);

    		

    		let repr:Repr<f32>=std::mem::transmute(buf2);
    		let ptr:*mut [f32;2]=std::mem::transmute(repr.ptr);
    		let size=repr.size/2;
            (ptr,size)
		};

        let game=conv::into_pointer(game);
		//let game:*mut MenuGame=std::mem::transmute(game);

        let mut col=[0.0f32;3];
        let colsptr:*mut f32=std::mem::transmute(&mut col[0]);

        let (verts_ptr,verts_size)={       
            let (buffptr,bufflen)={
                let buff:&mut [u8]=env.get_direct_buffer_address(jverts).unwrap();
                let k:ReprMut<u8>=unsafe{std::mem::transmute(buff)};
                (k.ptr,k.size)
            };

            let new_len=(bufflen/4)/2;
            //let kk:&mut [f32;2]=unsafe{std::mem::transmute(buffptr)};
            let data:&mut [[f32;2]]=unsafe{std::mem::transmute(Repr{ptr:buffptr,size:new_len})};


            let ff:ReprMut<[f32;2]>=unsafe{std::mem::transmute(data)};
            (ff.ptr,ff.size)
        };
		let val = rust_game_step(game,poses_ptr,poses_size,colsptr,verts_ptr,verts_size);

        if col.iter().fold(0.0,|acc,&x|acc+x)!=0.0{
            env.set_float_array_region(colors,0,&col);
        }

        if val{1}else{0}
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

    /*
    #[no_mangle]           
    pub unsafe extern "C" fn Java_kenreed_dinotreedemo_DinoGame_gameVerticies(env: JNIEnv, _: JClass,game:jlong,jbuff:JByteBuffer) {
        //let game:*mut MenuGame=std::mem::transmute(game);

        
        let game=conv::into_pointer(game);
      

        
        let (buffptr,bufflen)={
            let buff:&mut [u8]=env.get_direct_buffer_address(jbuff).unwrap();
            let k:ReprMut<u8>=unsafe{std::mem::transmute(buff)};
            (k.ptr,k.size)
        };

        let new_len=(bufflen/4)/2;
        //let kk:&mut [f32;2]=unsafe{std::mem::transmute(buffptr)};
        let data:&mut [[f32;2]]=unsafe{std::mem::transmute(Repr{ptr:buffptr,size:new_len})};


        let ff:ReprMut<[f32;2]>=unsafe{std::mem::transmute(data)};
        rust_game_verticies(game,ff.ptr,ff.size);
        /*
		
        let data:(*const [f32;2],usize)=rust_game_verticies(game);
        
        let repr:Repr<u8>=Repr{ptr:std::mem::transmute(data.0),size:data.1*2*std::mem::size_of::<f32>()};
        let buf:&mut [u8] =std::mem::transmute(repr);
        let ss=buf.len();
        let dest=env.get_direct_buffer_address(jbuff).unwrap();

        //TODO remove this copy!
        dest[0..ss].clone_from_slice(buf);
        
        //let bb=env.new_direct_byte_buffer(buf).unwrap();
        //std::mem::transmute(bb)
        */
    }
    */

}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
