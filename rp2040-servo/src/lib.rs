//! Servo PWM Builder
#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use {
    embassy_rp::pwm::{
        Config, 
        Pwm, 
        SetDutyCycle
    },
    embassy_time::Timer,
    {defmt_rtt as _, panic_probe as _},
};

const DEFAULT_SERVO_FREQ: u32 = 50; //Hertz
const DEFAULT_MIN_DUTY: u16 = 5; //us 
const DEFAULT_MAX_DUTY: u16 = 10;  //us
const DEFAULT_MAX_DEGREE_ROTATION: u16 = 180; //degree
const DEFAULT_INITIAL_POSITION: u16 = 0; //degree

pub struct ServoBuilder<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    freq: u32, 
    min_duty: u16,
    max_duty: u16,
    max_degree_rotation: u16,
    initial_position: u16,
}

impl<'d> ServoBuilder<'d> {
    pub fn new(pwm: Pwm<'d>) -> Self {
        Self {
            pwm,
            cfg: Config::default(),
            freq: DEFAULT_SERVO_FREQ,
            min_duty: DEFAULT_MIN_DUTY,
            max_duty: DEFAULT_MAX_DUTY,
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
            initial_position: DEFAULT_INITIAL_POSITION,
        }
    }

    pub fn set_servo_freq(mut self, freq: u32) -> Self {
        self.freq = freq;
        self
    }

    pub fn set_min_duty(mut self, duration: u16) -> Self {
        self.min_duty = duration;
        self
    }

    pub fn set_max_duty(mut self, duration: u16) -> Self {
        self.max_duty = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u16) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn set_initial_position(mut self, init_pos: u16) -> Self {
        self.initial_position = init_pos;
        self
    }

    pub fn build(mut self) -> Servo<'d> {
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 50u8;
        let period = (clock_freq_hz / (self.freq * divider as u32)) as u16 - 1;
        
        self.cfg.top = period;
        self.cfg.divider = divider.into();
        self.pwm.set_config(&self.cfg);

        Servo {
            pwm: self.pwm,
            cfg: self.cfg,
            period: period,
            min_duty_value: self.min_duty,
            max_duty_value: self.max_duty,
            max_degree_rotation:  self.max_degree_rotation,
            current_pos: self.initial_position,
        }
    }
}

pub struct Servo<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    period: u16,
    min_duty_value: u16,
    max_duty_value: u16,
    max_degree_rotation: u16,
    current_pos: u16,
}

#[allow(dead_code)]
impl<'d> Servo<'d> {
    fn set_current_pos(&mut self, degree: u16){
        self.current_pos = degree;
    }

    pub fn get_current_pos(&mut self) -> u16 {
        return self.current_pos;
    }

    pub fn enable(&mut self) {
        self.cfg.enable = true;
        self.pwm.set_config(&self.cfg);
        self.rotate(self.current_pos);
    }

    pub fn disable(&mut self) {
        self.cfg.enable = false;
        self.pwm.set_config(&self.cfg);
    }

    pub fn rotate(&mut self, degree: u16) {

        let mut duty = (((self.max_duty_value - self.min_duty_value) as u32 * degree as u32)/self.max_degree_rotation as u32) + self.min_duty_value as u32;
        
        if  duty > self.max_duty_value as u32{
            duty = self.max_duty_value as u32;
        }
        
        self.set_current_pos(degree);
        self.pwm.set_duty_cycle(duty as u16).unwrap();
    }

    pub async fn sweep(&mut self, target: u16, delay_ms: u64){
        let mut inc: i16 = 1;
        if self.current_pos > target {inc = -1}
    
        while self.current_pos != target {
            let mut new_pos = self.current_pos as i16 + inc;
            
            if new_pos<0 {new_pos = 0;}
            else if new_pos>180{new_pos = 180;}
    
            self.rotate(new_pos as u16);
            Timer::after_millis(delay_ms).await;
        }
    }

    fn rotate_duration(&mut self, duration: u16) {
        self.set_current_pos(duration.into());
        self.pwm.set_duty_cycle(duration).unwrap();
    }
}