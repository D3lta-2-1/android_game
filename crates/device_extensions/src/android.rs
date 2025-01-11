use std::sync::OnceLock;
use jni::{AttachGuard, JavaVM};
use jni::objects::{JObject, JValue, JValueGen, JValueOwned};
use winit::event_loop::ActiveEventLoop;
use winit::platform::android::ActiveEventLoopExtAndroid;
#[derive(Debug)]
struct JavaContext {
    java_vm: JavaVM,
    activity: JObject<'static>,
}

static JAVA_CONTEXT: OnceLock<JavaContext> = OnceLock::new();

pub struct DeviceExtensions {
    env: AttachGuard<'static>,
}

impl DeviceExtensions {

    fn setup(active_event_loop: &ActiveEventLoop) {
        let _  = JAVA_CONTEXT.get_or_init(|| {
            let android_app = active_event_loop.android_app();
            let activity = unsafe {
                JObject::from_raw(android_app.activity_as_ptr() as *mut _)
            };
            let java_vm = unsafe {
                JavaVM::from_raw(android_app.vm_as_ptr() as *mut _).unwrap()
            };
            JavaContext {
                java_vm,
                activity,
            }
        });
    }

    pub fn new(active_event_loop: &ActiveEventLoop) -> Self {
        Self::setup(active_event_loop);
        let env = JAVA_CONTEXT.get().unwrap().java_vm.attach_current_thread().unwrap();
        Self {
            env,
        }
    }

    pub fn vibrate(&mut self) {
        let string_content = JValueOwned::from(self.env.new_string("vibrator_manager").unwrap());

        let JValueGen::Object(vibrator_manager) = self.env.call_method(
            &JAVA_CONTEXT.get().unwrap().activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[string_content.borrow()]).unwrap() else { panic!("got a primitive") };

        let effect = self.env.call_static_method("android/os/VibrationEffect", "createPredefined", "(I)Landroid/os/VibrationEffect;", &[JValue::from(2)]).unwrap();

        let JValueGen::Object(vib) = self.env.call_method(
            vibrator_manager,
            "getDefaultVibrator",
            "()Landroid/os/Vibrator;",
            &[]).unwrap() else { panic!("got a primitive") };

        self.env.call_method(vib, "vibrate", "(Landroid/os/VibrationEffect;)V", &[effect.borrow()]).unwrap();
    }
}