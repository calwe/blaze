use core::{sync::atomic::Ordering, time};

use crate::{
    error,
    interrupts::{SLEEP_TICKS, U_SLEEP_TICKS},
    print, println, trace,
};

use super::rsdt::ACPISDTHeader;
use alloc::vec::Vec;
use bitfield::bitfield;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::hlt;

pub static mut GLOBAL_HPET: Option<Mutex<&'static HPET>> = None;

enum Registers64 {
    GeneralCapibilities = 0x00,
    GeneralConfiguration = 0x10,
    GeneralInterruptStatus = 0x20,
    MainCounterValue = 0xF0,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
struct AddressStructure {
    address_space_id: u8,
    register_bit_width: u8,
    register_bit_offset: u8,
    reserved: u8,
    address: u64,
}

bitfield! {
    #[derive(Clone, Copy, Default)]
    struct HPETFlags(u8);
    impl Debug;
    comparator_count, _: 4, 0;
    counter_size, _ : 5;
    legacy_replacement_route, _ : 7;
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
/// The HPET is the High Precision Event Timer.
/// https://wiki.osdev.org/HPET
///
/// ## Timer Layout
/// In the current implementation, Timer0 is used with oneshot mode
/// and Timer1 is used for periodic mode. Dynamic switching of this in
/// the future may be useful for systems with less timer comparitors.
pub struct HPET {
    header: ACPISDTHeader,
    hardware_rev_id: u8,
    flags: HPETFlags,
    pci_vendor_id: u16,
    address: AddressStructure,
    hpet_number: u8,
    minimum_tick: u16,
    page_protection: u8,
}

impl HPET {
    /// Creates a new HPET from a given address.
    pub fn new(addr: u32) -> &'static HPET {
        unsafe { &*(addr as *const HPET) }
    }

    pub fn init(&self) {
        let addr = self.address.address;
        trace!("HPET address: {:#x}", addr);
        let general_cap =
            GeneralCapibilities(self.read_register64(Registers64::GeneralCapibilities));
        let frequency = 10u64.pow(15) / general_cap.counter_clk_period();
        trace!("HPET frequency: {} Hz", frequency);
        let general_config =
            GeneralConfiguration(self.read_register64(Registers64::GeneralConfiguration));
        trace!("HPET general config: {:?}", general_config);

        let timer_count = general_cap.num_tim_cap() + 1;
        if timer_count <= 0 {
            panic!("No HPET timers found!");
        }
        let mut timer0 = TimerNConfiguration(self.read_n_config(0));
        let mut valid_interrupts = Vec::new();
        for i in 0..32 {
            if timer0.int_valid(i) {
                valid_interrupts.push(i);
            }
        }
        trace!("HPET timer 0 supports interrupts: {:?}", valid_interrupts);
        // configure timer 0
        for interrupt in valid_interrupts {
            match interrupt {
                2 | 8 => {}
                _ => {
                    trace!("Setting HPET timer 0 interrupt to {}", interrupt);
                    timer0.set_int_route_cnf(interrupt as u64);
                    break;
                }
            }
        }
        self.write_n_config(0, timer0.0);

        // ensure all timers are disabled
        for i in 0..timer_count {
            self.disable_n_timer(i as u8);
        }

        self.write_register64(Registers64::GeneralConfiguration, general_config.0 | 1);
    }

    // FIXME: Move this elsewhere
    /// sleep for a fixed duration, in ms
    pub fn sleep(&self, time_in_ms: u64) {
        SLEEP_TICKS.swap(time_in_ms, Ordering::Relaxed);
        self.periodic_init(1);
        while SLEEP_TICKS.load(Ordering::Relaxed) != 0 {
            hlt();
        }
        // unsafe {
        //     U_SLEEP_TICKS = time_in_ms;
        //     self.periodic_init(1);
        //     while U_SLEEP_TICKS != 0 {
        //         hlt();
        //     }
        // }
        self.disable_n_timer(1);
    }

    /// Use timer0 to send an interrupt after a set amount of time
    pub fn one_shot(&self, time_in_us: u64) {
        trace!("oneshot");
        let general_cap =
            GeneralCapibilities(self.read_register64(Registers64::GeneralCapibilities));
        let frequency = 10u64.pow(15) / general_cap.counter_clk_period();
        let current_counter = self.read_register64(Registers64::MainCounterValue);
        let time_in_ticks = (time_in_us as u64 * frequency) / 1000000;

        //trace!("Current count: {:x}", current_counter);
        self.disable_n_timer(0);
        self.write_n_comparator(0, current_counter + time_in_ticks);
        self.enable_n_timer(0);
    }

    pub fn periodic_init(&self, time_in_ms: u64) {
        trace!("Setting up periodic mode");
        let general_cap =
            GeneralCapibilities(self.read_register64(Registers64::GeneralCapibilities));
        if general_cap.num_tim_cap() < 1 {
            error!("No second timer!");
            return;
        }
        let mut timer1_conf = TimerNConfiguration(self.read_n_config(1));
        if !timer1_conf.per_int_cap() {
            error!("Timer doesnt support periodic mode!");
            return;
        }

        self.disable_n_timer(1);
        let mut valid_interrupts = Vec::new();
        for i in 0..32 {
            if timer1_conf.int_valid(i) {
                valid_interrupts.push(i);
            }
        }
        trace!("HPET timer 1 supports interrupts: {:?}", valid_interrupts);
        // configure timer 0
        for interrupt in valid_interrupts {
            match interrupt {
                // TODO: Check what other timers use, not hardcode 16
                2 | 8 | 16 => {}
                _ => {
                    trace!("Setting HPET timer 1 interrupt to {}", interrupt);
                    timer1_conf.set_int_route_cnf(interrupt as u64);
                    break;
                }
            }
        }
        timer1_conf.set_init_enb_cnf(true);
        timer1_conf.set_type_cnf(true);
        timer1_conf.set_val_set_cnf(true);
        self.write_n_config(1, timer1_conf.0);

        let frequency = 10u64.pow(15) / general_cap.counter_clk_period();
        trace!("Frequency: {} MHz", frequency / 1_000_000);
        let current_counter = self.read_register64(Registers64::MainCounterValue);
        // time in ms (currently 1000ms)
        let time_in_ticks = (time_in_ms * frequency) / 1000;
        trace!("Ticks: {time_in_ticks}");

        self.write_n_comparator(1, current_counter + time_in_ticks);
        self.write_n_comparator(1, time_in_ticks);
        //trace!("comp: {} - {}", current_counter, self.read_n_comparator(1));

        trace!("{:?}", timer1_conf);
    }

    pub fn disable_n_timer(&self, timer: u8) {
        let mut tim = TimerNConfiguration(self.read_n_config(timer));
        tim.set_init_enb_cnf(false);
        self.write_n_config(timer, tim.0);
    }

    fn enable_n_timer(&self, timer: u8) {
        let mut tim = TimerNConfiguration(self.read_n_config(timer));
        tim.set_init_enb_cnf(true);
        self.write_n_config(timer, tim.0);
    }

    fn read_register64(&self, register: Registers64) -> u64 {
        let addr = self.address.address;
        let addr = addr + register as u64;
        // trace!("Reading HPET register {:#x}", addr);
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }

    fn write_register64(&self, register: Registers64, value: u64) {
        let addr = self.address.address;
        let addr = addr + register as u64;
        // trace!("Writing HPET register {:#x} {:#x}", addr, value);
        unsafe { core::ptr::write_volatile(addr as *mut u64, value) }
    }

    fn read_n_config(&self, n: u8) -> u64 {
        let addr = self.address.address;
        let addr = addr + 0x100 + (n as u64 * 0x20);
        //trace!("Reading HPET timer {} config {:#x}", n, addr);
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }

    fn write_n_config(&self, n: u8, value: u64) {
        let addr = self.address.address;
        let addr = addr + 0x100 + (n as u64 * 0x20);
        //trace!("Writing HPET timer {} config {:#x} {:#x}", n, addr, value);
        unsafe { core::ptr::write_volatile(addr as *mut u64, value) }
    }

    fn read_n_comparator(&self, n: u8) -> u64 {
        let addr = self.address.address;
        let addr = addr + 0x108 + (n as u64 * 0x20);
        trace!("Reading HPET timer {} comparator {:#x}", n, addr);
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }

    fn write_n_comparator(&self, n: u8, value: u64) {
        let addr = self.address.address;
        let addr = addr + 0x108 + (n as u64 * 0x20);
        trace!(
            "Writing HPET timer {} comparator {:#x} {:#x}",
            n,
            addr,
            value
        );
        unsafe { core::ptr::write_volatile(addr as *mut u64, value) }
    }
}

// Registers

bitfield! {
    struct GeneralCapibilities(u64);
    impl Debug;
    rev_id, _: 7, 0;
    num_tim_cap, _: 12, 8;
    counter_size_cap, _: 13;
    legacy_replacement_route_cap, _: 15;
    pci_vendor_id, _: 31, 16;
    counter_clk_period, _: 63, 32;
}

bitfield! {
    struct GeneralConfiguration(u64);
    impl Debug;
    enable_cnf, _: 0;
    legacy_replacement_route_cnf, _: 1;
}

bitfield! {
    struct TimerNConfiguration(u64);
    impl Debug;
    int_type_cnf, set_int_type_cnf: 1;
    init_enb_cnf, set_init_enb_cnf: 2;
    type_cnf, set_type_cnf: 3;
    per_int_cap, set_per_int_cap: 4;
    size_cap, set_size_cap: 5;
    val_set_cnf, set_val_set_cnf: 6;
    mode32_cnf, set_mode32_cnf: 8;
    int_route_cnf, set_int_route_cnf: 13, 9;
    fsb_en_cnf, set_fsb_en_cnf: 14;
    fsb_int_del_cap, set_fsb_int_del_cap: 15;
    int_route_cap, set_int_route_cap: 63, 32;
}

impl TimerNConfiguration {
    fn int_valid(&self, n: u8) -> bool {
        self.int_route_cap() & (1 << n) != 0
    }
}

// TODO: The rest of them...
