use std::{io::IoSliceMut, time::{Duration, Instant}};

use nix::{sys::uio::{RemoteIoVec, process_vm_readv}, unistd::Pid};

use egui::{menu, CentralPanel, Context, Grid, ScrollArea, Slider, TextEdit, TopBottomPanel, ViewportBuilder, ViewportCommand};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([600.0, 400.0])
        ,
        ..Default::default()       
    };
    eframe::run_native(
        "Examine Memory",
        native_options,
        Box::new(|cc| Box::new(XApp::new(cc)))
    )
}

pub struct XApp {
    memory_address: String,
    start_address: Option<usize>,
    validation_message: String,
    num_addresses: usize,
    pid: String,
    data32: Vec<i32>,
    last_update: Instant,
    update_interval: Duration,
}

impl Default for XApp {
    fn default() -> Self {
        Self {
            memory_address: "0xD34DB33F0".to_owned(),
            start_address: None,
            validation_message: String::new(),
            num_addresses: 10,
            pid: String::from(""),
            data32: Vec::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
        }
    }
}

impl XApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize look here
        
        Default::default()
    }
}

impl eframe::App for XApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter memory address (e.g., 0xd34db33f):");
            ui.add(
                TextEdit::singleline(&mut self.memory_address)
                    .hint_text("0xd34db33f")
                    .desired_width(200.0),
            );

            ui.label("Enter process ID:");
            ui.add(
                TextEdit::singleline(&mut self.pid)
                    .hint_text("1234")
                    .desired_width(200.0),
            );

            ui.add(Slider::new(&mut self.num_addresses, 1..=500).text("Addresses"));

            if self.memory_address.starts_with("0x") && self.memory_address.len() >= 8 {
                match usize::from_str_radix(&self.memory_address[2..], 16) {
                    Ok(address) => {
                        self.start_address = Some(address);
                        self.validation_message = "Valid memory address".to_string();
                    },
                    Err(_) => {
                        self.start_address = None;
                        self.validation_message = "Invalid memory address".to_string();
                    },
                }
            } else {
                self.validation_message = "Invalid memory address format".to_string();
            }

            ui.label(&self.validation_message);

            if let Some(address) = self.start_address {
                ui.label(format!("Saved memory address: 0x{:X}", address));
                ui.separator();

                let now = Instant::now();
                if now.duration_since(self.last_update) >= self.update_interval {
                    self.last_update = now;
                    let pid = Pid::from_raw(self.pid.parse().unwrap());
                    let mut data = vec![0u8; self.num_addresses as usize * 4];
                    let local_iov = IoSliceMut::new(&mut data);
                    let remote_iov = RemoteIoVec {
                        base: address,
                        len: self.num_addresses as usize * 4,
                    };
                    match process_vm_readv(pid, &mut [local_iov], &[remote_iov]) {
                        Ok(_) => {
                            self.data32 = data.chunks_exact(4).map(|chunk| {
                                i32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                            }).collect();
                        },
                        Err(_) => self.data32.clear(),
                    }
                }

                ui.label("Memory Address Table:");
                ScrollArea::vertical().show(ui, |ui| {
                    Grid::new("memory_address_table").striped(true).show(ui, |ui| {
                        ui.label("Address");
                        ui.label("Hex32");
                        ui.label("Dec32");
                        ui.label("Hex64");
                        ui.label("Dec64");
                        ui.end_row();

                        for i in 0..self.num_addresses {
                            ui.label(format!("0x{:X}", address + i * 4));
                            // Todo: make this safe
                            let hex32 = self.data32.get(i).unwrap_or(&0);
                            ui.label(format!("0x{:X}", hex32));
                            ui.label(format!("{}", hex32));
                            // useless rn
                            let hex64 = self.data32.get(i).unwrap_or(&0);
                            ui.label(format!("0x{:X}", hex64));
                            ui.end_row();
                        }
                    });
                });
            }
        });
    }
}