
extern crate vk;

fn main() {

	let instance = vk::InstanceBuilder {
		application_name: "hello triangle".to_string(),
		.. Default::default()
	}.create_instance().expect("Couldn't create instance");

	let wsi_info = instance.create_wsi(instance.vk(), 800, 600);

	let device;
	let graphics_queue;
	let compute_queue;
	let transfer_queue;
	{
		let mut db = vk::DeviceBuilder::new(&instance);
		device = db.default_queues_physical_device(&wsi_info.0).create_device().expect("Couldn't create logical device");

		println!("Using device index {:?}, graphics, compute, transfer queue family inices: {}, {}, {}",
			db.physical_device.unwrap().1, db.queue_create_infos[0].0, db.queue_create_infos[1].0, db.queue_create_infos[2].0);

		graphics_queue = device.get_queue(db.queue_create_infos[0].0, 0);
		compute_queue = device.get_queue(db.queue_create_infos[1].0, 0);
		transfer_queue = device.get_queue(db.queue_create_infos[2].0, 0);
	}

	let mem = vk::MemoryAllocator::new(&instance);
	struct GPUData {
		index: u32,
		data: (f32, f32, f32, f32)
	}
	let mut data = mem.create_mem();
	{
		{
			let mut mapped = data.map();
			let ptr = mapped.get_ptr::<GPUData>();
			unsafe {
			//(*ptr).index = 123;
			//(*ptr).data = (1.0, 2.0, 3.0, 4.0);
			}
		}
		mem.upload(data);
	}
	/*let mut data = mem.create_mem_typed_safe::<GPUData>(GPUData{index:0,data:(1.0, 1.0, 1.0, 1.0)});
	{
		let mut mapped: Box<GPUData> = data.map();
		//mapped.index = 123;
		//mapped.data = (1.0, 2.0, 3.0, 4.0);
		mem.upload_typed_safe(data);
	}*/

	/*let gpu_data = mem.upload(transfer_queue, data);
	let gpu_texture = mem.upload_texture_from_file("x.ktx");
	let gpu_texture_view = vk::MemoryView::new(instance, gpu_texture);*/
}
