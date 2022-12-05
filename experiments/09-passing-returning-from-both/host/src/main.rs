/*

Problem statement:
- Call a guest function by passing a string
  - This guest function will then read this string
  - And call a host function again by passing some string
  - Return a string back to guest
  - Return a string back to host
*/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new_async(&mut store, &module, &[]).await?;

    // Calling Guest Function
    call_guest(&mut store, &instance, "Hello From Host").await?;

    // let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    // let size = 10;
    // let memory_address = alloc.call_async(&mut store, size as u32).await?;
    // println!("Wasm Memory address: {}", memory_address);
    // let input = vec![1 as u8, 2, 3, 4, 5, 6, 7, 8 ,9, 10];
    // println!("Coping the data into the wasm memory");
    // memory.write(&mut store, memory_address as usize, input.as_ref()).unwrap();
    // println!("data copied successfully");
    //
    // let array_sum = instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "array_sum")?;
    // let sum_of_array = array_sum.call_async(&mut store, (memory_address, size)).await?;
    // println!("Array Sum: {}", sum_of_array);
    //
    // // println!("Deallocating from the wasm memory");
    // // println!("Coping the data into the wasm memory");
    //
    // // let sum_of_array = array_sum.call_async(&mut store, (memory_address, size)).await?;
    // // println!("Array Sum: {}", sum_of_array);
    //
    // let dealloc = instance.get_typed_func::<(u32, u32), (), _>(&mut store, "dealloc")?;
    // dealloc.call_async(&mut store, (memory_address, size)).await?;
    // println!("Memory deallocated");

    Ok(())
}



// Sending data to guest
// let's say we are sending a string
// write that data to wasm memory using alloc function
// and pass the pointer


async fn call_guest(mut store: &mut wasmtime::Store<()>, instance: &wasmtime::Instance, message: &str) -> anyhow::Result<()> {
    let memory: wasmtime::Memory = instance.get_memory(&mut store, "memory").unwrap();
    let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    let data = message.as_bytes();
    let memory_address = alloc.call_async(&mut store, data.len() as u32).await?;
    println!("Writing Rust Data to WASM memory: {}", message);
    memory.write(&mut store, memory_address as usize, data)?;
    println!("Calling guest function with memory pointer: {}", memory_address);
    let data = utils::SizedData {
        len: data.len() as u32,
        data: memory_address
    };
    let call_guest = instance.get_typed_func::<(u32), u32, _>(&mut store, "call_guest")?;
    let guest_memory = call_guest.call_async(&mut store, (data.to_bytes())).await?;
    println!("Guest Memory Address: {}", guest_memory);

    Ok(())
}

// This utility can be shared at the both end
mod utils {

    pub struct SizedData {
        pub len: u32,
        pub data: u32,
    }

    impl SizedData {
        pub fn from_string(s: String) -> Self {
            let mut data: Vec<u8> = s.into_bytes();
            let len = data.len() as u32;
            let data_ptr = data.as_mut_ptr() as u32;
            // While returning from here rust will not free the memory
            // Otherwise rust will free the memory while returning because of the ownership
            // of the variable
            std::mem::forget(data);
            return SizedData {
                data: data_ptr,
                len,
            };
        }

        pub fn to_bytes(self) -> u32 {
            // pointer array which contains 64 bits
            // in this array we will store the data pointer and len value
            // both are 32 bit
            let mut pointer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

            // Return the memory representation of this integer as a byte array in native byte order.
            // Note: Wasm follows little endian architecture
            let data_pointer_bytes = self.data.to_ne_bytes();
            let len_value_bytes = self.len.to_ne_bytes();

            // Now we will store both the values in the pointer array and
            // will return that array pointer to rust

            // storing data pointer
            pointer[0] = data_pointer_bytes[0];
            pointer[1] = data_pointer_bytes[1];
            pointer[2] = data_pointer_bytes[2];
            pointer[3] = data_pointer_bytes[3];

            // storing length of the data
            pointer[4] = len_value_bytes[0];
            pointer[5] = len_value_bytes[1];
            pointer[6] = len_value_bytes[2];
            pointer[7] = len_value_bytes[3];

            // To tell rust, forget this memory and
            // do not clean it while returning from the function
            std::mem::forget(pointer);
            pointer.as_ptr() as u32
        }
    }
}


