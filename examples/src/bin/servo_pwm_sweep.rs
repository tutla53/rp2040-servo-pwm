//! Servo Task

#![no_std]
#![no_main]

use {
    rp2040_servo::ServoBuilder,
    embassy_executor::Spawner,
    embassy_rp::{
        bind_interrupts,
        config::Config,
        pwm::{Config as PwmConfig, Pwm},
        usb::{Driver, InterruptHandler as UsbInterruptHandler},
        peripherals::USB,
    },
    embassy_time::Timer,
};

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});


#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner){
    let p = embassy_rp::init(Config::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let servo_pwm_device = Pwm::new_output_a(p.PWM_SLICE5, p.PIN_10, PwmConfig::default());
    
    let mut servo_motor = ServoBuilder::new(servo_pwm_device)
        .set_servo_freq(50)
        .set_max_degree_rotation(0)
        .set_min_duty(2100)
        .set_max_duty(8200)
        .set_initial_position(180)
        .build();

    servo_motor.enable();
    Timer::after_secs(1).await;
    let mut target: u32 = 0;
    let inc: u32 = 1;
    let delay_ms: u64 = 1;
    let mut current_duty = servo_motor.get_current_duty();
    let mut target_duty = servo_motor.degree_to_duty(target);
    log::info!("Current Pos: {}", servo_motor.get_current_pos());

    loop {

        current_duty = servo_motor.get_current_duty();
        target_duty = servo_motor.degree_to_duty(target);
        log::info!("Current Pos {} - Target {}", servo_motor.get_current_pos(), target);
        log::info!("Current Duty {} - Target {}", current_duty, target_duty);
        
        
        log::info!("Waiting the servo to sweep....");
        if current_duty > target_duty {
            while (current_duty > target_duty) && (current_duty >= servo_motor.get_min_duty()) && (current_duty <= servo_motor.get_max_duty()) {
                servo_motor.rotate_duty(current_duty);
                current_duty = current_duty - inc;
                log::info!("Current Duty {} - Target {}", current_duty, target_duty);
                Timer::after_millis(delay_ms).await;
            }
        }

        else if current_duty < target_duty {
            while (current_duty < target_duty) && (current_duty >= servo_motor.get_min_duty()) && (current_duty <= servo_motor.get_max_duty()) {
                servo_motor.rotate_duty(current_duty);                
                current_duty = current_duty + inc;
                log::info!("Current Duty {} - Target {}", current_duty, target_duty);
                Timer::after_millis(delay_ms).await;
            }
        }
        
        log::info!("Servo Sweep is Complete\n");
        if target == 0 {target = 180;}
        else {target = 0};

        Timer::after_millis(10).await;
    }
}
