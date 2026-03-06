use std::sync::{Arc, Mutex};

use crate::app::submenu_enum::SubmenuP2pool;
use crate::app::{App, AppEgui, Tab};
use crate::components::node::RemoteNodes;
#[cfg(not(feature = "distro"))]
use crate::errors::{ErrorButtons, ErrorFerris};
use crate::helper::{Helper, ProcessName, ProcessState};
use crate::inits::init_text_styles;
#[cfg(not(feature = "distro"))]
use crate::utils::errors::WarnUpdateData;
use crate::{NODE_MIDDLE, P2POOL_MIDDLE, SECOND, XMRIG_MIDDLE, XMRIG_PROXY_MIDDLE, XVB_MIDDLE};
use derive_more::derive::{Deref, DerefMut};
use log::debug;

impl eframe::App for AppEgui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut app = self.inner.lock();
        // *-------*
        // | DEBUG |
        // *-------*
        if mitigate_wgpu_mem_leak(ui.ctx()) {
            return;
        }
        debug!("App | ----------- Start of [update()] -----------");
        // If closing
        app.quit(ui.ctx());
        // Handle Keys
        let (key, wants_input) = app.keys_handle(ui.ctx());

        // Refresh AT LEAST once a second
        debug!("App | Refreshing frame once per second");
        ui.ctx().request_repaint_after(SECOND);

        // Get P2Pool/XMRig process state.
        // These values are checked multiple times so
        // might as well check only once here to save
        // on a bunch of [.lock().unwrap()]s.
        let mut process_states = ProcessStatesGui::new(&app);
        // resize window and fonts if button "set" has been clicked in Gupax tab
        if app.must_resize {
            init_text_styles(ui.ctx(), app.state.gupax.selected_scale);
            app.must_resize = false;
        }
        // check for windows that a local instance of xmrig is not running outside of Gupax. Important because it could lead to crashes on this platform.
        // Warn only once per restart of Gupax.
        #[cfg(target_os = "windows")]
        if !app.xmrig_outside_warning_acknowledge
            && ProcessName::Xmrig
                .is_process_running(&mut app.helper.lock().unwrap().sys_info.lock().unwrap())
            && !process_states.find(ProcessName::Xmrig).alive
        {
            app.error_state.set("An instance of xmrig is running outside of Gupax.\nThis is not supported and could lead to crashes on this platform.\nPlease stop your local instance and start xmrig from Gupax Xmrig tab.", ErrorFerris::Error, ErrorButtons::Okay);
            app.xmrig_outside_warning_acknowledge = true;
        }

        #[cfg(not(feature = "distro"))]
        app.ask_download_binaries();
        // If there's an error, display [ErrorState] on the whole screen until user responds
        debug!("App | Checking if there is an error in [ErrorState]");
        if app.error_state.error {
            app.quit_error_panel(ui, &process_states, &key);
            return;
        }
        // Compare [og == state] & [node_vec/pool_vec] and enable diff if found.
        // The struct fields are compared directly because [Version]
        // contains Arc<Mutex>'s that cannot be compared easily.
        // They don't need to be compared anyway.
        debug!("App | Checking diff between [og] & [state]");
        let og = app.og.lock().unwrap();
        let diff = og.status != app.state.status
            || og.gupax != app.state.gupax
            || og.node != app.state.node
            || og.p2pool != app.state.p2pool
            || og.xmrig != app.state.xmrig
            || og.xmrig_proxy != app.state.xmrig_proxy
            || og.xvb != app.state.xvb
            || app.og_node_vec != app.node_vec
            || app.og_pool_vec != app.pool_vec;
        drop(og);
        app.diff = diff;

        let mut selected_nodes = None;
        // crawl/pinged/selected remote node refresh
        if app.state.gupax.auto.crawl || app.tab == Tab::P2pool {
            let mut crawler_lock = app.crawler.lock().unwrap();
            let mut ping_lock = app.ping.lock().unwrap();
            let crawling = crawler_lock.crawling;
            let ping_nodes = &mut ping_lock.nodes;
            let crawl_nodes = &mut crawler_lock.nodes;

            if *ping_nodes != *crawl_nodes && !crawl_nodes.is_empty() {
                *ping_nodes = crawl_nodes.clone();
                if !crawling {
                    *crawl_nodes = RemoteNodes::default();
                }
            }

            // refresh the selected node with the fastest from the pinged nodes if it was empty
            if app.state.p2pool.selected_remote_node.is_none() {
                selected_nodes = ping_nodes.first().cloned();
            }
        }
        if (app.state.gupax.auto.crawl || app.tab == Tab::P2pool)
            && app.state.p2pool.selected_remote_node.is_none()
        {
            app.state.p2pool.selected_remote_node = selected_nodes;
        }
        // replace backup host by custom ones when user is in p2pool advanced sub menu
        // Only if the backup host is different from the custom ones
        if app.state.p2pool.submenu != SubmenuP2pool::Advanced && app.tab == Tab::P2pool {
            let mut backup_hosts = app.backup_hosts.lock().unwrap();
            if app.node_vec.iter().any(|(_, n)| backup_hosts.contains(n)) {
                *backup_hosts = app.node_vec.iter().map(|n| n.1.clone()).collect();
            }
        }

        app.top_panel(ui);
        app.bottom_panel(ui, &key, wants_input, &process_states);
        // xvb_is_alive is not the same for bottom and for middle.
        // for status we don't want to enable the column when it is retrying requests.
        // but also we don't want the user to be able to start it in this case.
        let p_xvb = process_states.find_mut(ProcessName::Xvb);
        p_xvb.alive = p_xvb.state != ProcessState::Dead;
        app.middle_panel(ui, key, &process_states);
    }
}
#[derive(Debug)]
pub struct ProcessStateGui {
    pub name: ProcessName,
    pub state: ProcessState,
    pub alive: bool,
    pub waiting: bool,
}

impl ProcessStateGui {
    pub fn run_middle_msg(&self) -> &str {
        match self.name {
            ProcessName::Node => NODE_MIDDLE,
            ProcessName::P2pool => P2POOL_MIDDLE,
            ProcessName::Xmrig => XMRIG_MIDDLE,
            ProcessName::XmrigProxy => XMRIG_PROXY_MIDDLE,
            ProcessName::Xvb => XVB_MIDDLE,
        }
    }
    pub fn stop(&self, helper: &Arc<Mutex<Helper>>) {
        match self.name {
            ProcessName::Node => Helper::stop_node(helper),
            ProcessName::P2pool => Helper::stop_p2pool(helper),
            ProcessName::Xmrig => Helper::stop_xmrig(helper),
            ProcessName::XmrigProxy => Helper::stop_xp(helper),
            ProcessName::Xvb => Helper::stop_xvb(helper),
        }
    }
}

#[derive(Deref, DerefMut, Debug)]
pub struct ProcessStatesGui(Vec<ProcessStateGui>);

impl ProcessStatesGui {
    // order is important for lock
    pub fn new(app: &App) -> Self {
        let mut process_states = ProcessStatesGui(vec![]);
        for process in [
            &app.node,
            &app.p2pool,
            &app.xmrig,
            &app.xmrig_proxy,
            &app.xvb,
        ] {
            let lock = process.lock().unwrap();
            process_states.push(ProcessStateGui {
                name: lock.name,
                alive: lock.is_alive(),
                waiting: lock.is_waiting(),
                state: lock.state,
            });
        }
        process_states
    }
    pub fn is_alive(&self, name: ProcessName) -> bool {
        self.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("This vec should always contains all Processes {self:?}"))
            .alive
    }
    pub fn find(&self, name: ProcessName) -> &ProcessStateGui {
        self.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("This vec should always contains all Processes {self:?}"))
    }
    pub fn find_mut(&mut self, name: ProcessName) -> &mut ProcessStateGui {
        self.iter_mut()
            .find(|p| p.name == name)
            .expect("This vec should always contains all Processes")
    }
}

/// Helper function to mitigate https://github.com/emilk/egui/issues/7434.
///
/// If this returns true, the app should early return in the `update()` function
/// or call `wgpu::Device::poll()`
fn mitigate_wgpu_mem_leak(ctx: &egui::Context) -> bool {
    let mut is_minimized = false;
    ctx.input(|reader| {
        is_minimized = reader.viewport().minimized.unwrap_or_default();
    });

    is_minimized
}

impl App {
    /// ask the user if he wants gupax to download the required binaries
    /// Will not ask if every path of binaries exist or if he checked the "do not check next time".
    #[cfg(not(feature = "distro"))]
    pub fn ask_download_binaries(&mut self) {
        if !self.ask_download_start_acknowledge && self.state.gupax.updates.ask_download_start {
            let p2pool_exist = self.state.gupax.absolute_p2pool_path.is_file();
            let node_exist = self.state.gupax.absolute_node_path.is_file();
            let xmrig_exist = self.state.gupax.absolute_xmrig_path.is_file();
            let xp_exist = self.state.gupax.absolute_xp_path.is_file();
            if !p2pool_exist || !node_exist || !xmrig_exist || !xp_exist {
                let msg = format!(
                    "Gupax is missing the binary of:\n{}\n{}\n{}\n{}\n\nDo you want it to download them now ?",
                    if !p2pool_exist { "P2Pool" } else { "" },
                    if !node_exist { "Node" } else { "" },
                    if !xmrig_exist { "XMRig" } else { "" },
                    if !xp_exist { "XMRig-Proxy" } else { "" }
                );
                let mut binaries = vec![];
                if !p2pool_exist {
                    binaries.push("p2pool".to_string());
                }
                if !node_exist {
                    binaries.push("monerod".to_string());
                }
                if !xmrig_exist {
                    binaries.push("xmrig".to_string());
                }
                if !xp_exist {
                    binaries.push("xmrig-proxy".to_string());
                }
                self.error_state.set(
                    msg,
                    ErrorFerris::Cute,
                    ErrorButtons::WarnUpdate(WarnUpdateData {
                        yes_button: "Download missing binaries".to_string(),
                        no_button: "No, and do not ask again".to_string(),
                        name: binaries.join(" "),
                    }),
                );
            }
        }
        // only check once at start
        self.ask_download_start_acknowledge = true;
    }
}
