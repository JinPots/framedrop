// Use the global Tauri API for maximum reliability across different bundler setups
const tauri = window.__TAURI__ || {};
const { invoke } = tauri.core || { invoke: async () => {} };
const { listen } = tauri.event || { listen: async () => {} };
const { open }   = tauri.dialog || { open: async () => {} };

document.addEventListener('DOMContentLoaded', async () => {

  // ── State ──────────────────────────────────────────────
  let currentState = 'idle'; // 'idle' | 'copying'
  let currentView  = 'dashboard';
  let config       = null;
  let copyProgress = null;     // latest ProgressPayload
  let detectedCard = null;     // latest SDCardPayload
  let history      = [];       // ingest sessions

  // Load persisted config from backend
  try { 
    config = await invoke('get_config'); 
  } catch(e) { 
    console.warn('Config load failed', e); 
  }
  if (!config) {
    config = { 
      destPath:'', organizeDate:true, organizeBrand:true, 
      includeRaw:true, includeJpeg:true, includeVideo:true, 
      syncRemote:false, remoteIp:'', remotePath:'', remoteMethod:'SMB',
      notifyTray:true, playSound:false, 
      startWithOs:false, minimizeToTray:true, autoIngest:false,
      webhookUrl:'', webhookEnabled:false, webhookPingId:'',
      language: 'en',
      separateVideo: true,
      scanVideoDirs: true,
      dateSource: 'capture',
      videoFolder: 'separate'
    };
  }

  const translations = {
    en: {
      dashboard: "Dashboard",
      history: "History",
      settings: "Settings",
      idle: "Idle",
      copying: "Copying...",
      watching_cards: "Watching for SD cards...",
      plug_in_card: "Plug in a camera SD card to start automatic ingestion. FrameDrop will handle the organization for you.",
      manual_ingest: "Manual Ingest...",
      open_destination: "Open Destination",
      sd_card_found: "SD Card: ",
      ready_to_begin: "Found {count} files on drive {path}. Ready to begin ingestion.",
      start_ingest: "Start Ingest",
      select_different: "Select Different Folder",
      copying_from: "Copying from ",
      files_counter: "{current} of {total} files",
      cancel: "Cancel",
      time_remaining: "Time remaining",
      speed: "Speed",
      current_file: "Current File",
      recent_transfers: "Recent Transfers",
      no_history: "No ingest history yet. Plug in an SD card to get started.",
      recent_ingests: "Recent Ingests",
      clear_history: "Clear History",
      open_folder: "Open Folder",
      dest_path: "Destination Path",
      browse: "Browse",
      open_explorer: "Open in Explorer",
      folder_structure: "Folder Structure",
      org_date: "Organize by date",
      org_date_desc: "Creates YYYY-MM-DD subfolders",
      org_model: "Organize by camera model",
      org_model_desc: "Creates EOS R5, Z9, etc. subfolders",
      live_preview: "Live Preview",
      remote_sync: "Remote Sync",
      sync_desc: "Sync to remote PC after ingest",
      remote_ip: "Remote IP / Hostname",
      trans_method: "Transfer Method",
      remote_dest: "Remote Destination Path",
      test_conn: "Test Connection",
      webhook_title: "Webhook Notification (Discord / Telegram)",
      webhook_desc: "Send summary to Discord or Telegram after ingest",
      enable_webhook: "Enable Webhook Notifications",
      webhook_url: "Webhook URL",
      test: "Test",
      ping_id: "User ID to Ping (Optional)",
      file_types: "File Types",
      raw_files: "RAW files",
      jpg_files: "JPEG files",
      vid_files: "Video files",
      notifs: "Notifications",
      tray_notif: "System tray notification",
      sound_complete: "Sound on completion",
      app_behavior: "App Behavior",
      start_win: "Start with Windows",
      min_close: "Minimize on close",
      auto_promptless: "Auto-ingest promptless",
      toast_dest_first: "Please set a destination path in Settings first.",
      toast_ingest_err: "Ingest error: ",
      done: "Done!",
      copied_skipped: "{copied} copied, {skipped} skipped.",
      photos: "Photos",
      raw: "RAW Photos",
      videos: "Videos",
      duplicates: "Duplicates",
      sep_video: "Separate videos into Video folder",
      sep_video_desc: "Creates a /Video subfolder for all video files",
      scan_video_dirs: "Scan common camera video structures",
      scan_video_dirs_desc: "Detects PRIVATE/AVCHD, M4ROOT, etc.",
      date_source: "Date Source",
      date_capture: "Capture Date (from metadata)",
      date_import: "Import Date (transfer time)",
      vid_location: "Video Subfolder",
      vid_separate: "Separate Video Folder (/Video)",
      vid_mixed: "Mixed with Photos (as if same card)",
    },
    vi: {
      dashboard: "Bảng điều khiển",
      history: "Lịch sử",
      settings: "Cài đặt",
      idle: "Chờ",
      copying: "Đang chép...",
      watching_cards: "Đang chờ thẻ nhớ...",
      plug_in_card: "Cắm thẻ nhớ máy ảnh để bắt đầu nhập ảnh tự động. FrameDrop sẽ tự động phân loại cho bạn.",
      manual_ingest: "Nhập thủ công...",
      open_destination: "Mở thư mục đích",
      sd_card_found: "Thẻ nhớ: ",
      ready_to_begin: "Tìm thấy {count} tệp trên ổ {path}. Sẵn sàng nhập ảnh.",
      start_ingest: "Bắt đầu nhập",
      select_different: "Chọn thư mục khác",
      copying_from: "Đang chép từ ",
      files_counter: "{current} trên {total} tệp",
      cancel: "Hủy bỏ",
      time_remaining: "Thời gian còn lại",
      speed: "Tốc độ",
      current_file: "Tệp hiện tại",
      recent_transfers: "Lịch sử chép gần đây",
      no_history: "Chưa có lịch sử nhập ảnh. Cắm thẻ nhớ để bắt đầu.",
      recent_ingests: "Lịch sử nhập ảnh",
      clear_history: "Xóa lịch sử",
      open_folder: "Mở thư mục",
      dest_path: "Đường dẫn đích",
      browse: "Duyệt tệp",
      open_explorer: "Mở trong Explorer",
      folder_structure: "Cấu trúc thư mục",
      org_date: "Phân loại theo ngày",
      org_date_desc: "Tạo thư mục YYYY-MM-DD",
      org_model: "Phân loại theo dòng máy",
      org_model_desc: "Tạo thư mục EOS R5, Z9, v.v.",
      live_preview: "Xem trước",
      remote_sync: "Đồng bộ từ xa",
      sync_desc: "Đồng bộ lên PC sau khi nhập",
      remote_ip: "IP / Hostname máy xa",
      trans_method: "Phương thức truyền",
      remote_dest: "Đường dẫn đích máy xa",
      test_conn: "Kiểm tra kết nối",
      webhook_title: "Thông báo Webhook (Discord / Telegram)",
      webhook_desc: "Gửi tóm tắt lên Discord hoặc Telegram",
      enable_webhook: "Bật thông báo Webhook",
      webhook_url: "Đường dẫn Webhook",
      test: "Kiểm tra",
      ping_id: "ID Người dùng để nhắc (Tùy chọn)",
      file_types: "Loại tệp",
      raw_files: "Tệp RAW",
      jpg_files: "Tệp JPEG",
      vid_files: "Tệp Video",
      notifs: "Thông báo",
      tray_notif: "Thông báo thanh tác vụ",
      sound_complete: "Âm thanh khi hoàn tất",
      app_behavior: "Hành vi ứng dụng",
      start_win: "Khởi động cùng Windows",
      min_close: "Thu nhỏ khi đóng",
      auto_promptless: "Tự động nhập không cần hỏi",
      toast_dest_first: "Vui lòng thiết lập đường dẫn đích trong Cài đặt.",
      toast_ingest_err: "Lỗi nhập ảnh: ",
      done: "Hoàn tất!",
      copied_skipped: "Đã chép {copied}, bỏ qua {skipped}.",
      photos: "Ảnh JPEG",
      raw: "Ảnh RAW",
      videos: "Video",
      duplicates: "Trùng lặp",
      sep_video: "Tách Video vào thư mục riêng",
      sep_video_desc: "Tạo thư mục /Video cho tất cả các tệp video",
      scan_video_dirs: "Quét các cấu trúc video máy ảnh",
      scan_video_dirs_desc: "Phát hiện PRIVATE/AVCHD, M4ROOT, v.v.",
      date_source: "Nguồn ngày",
      date_capture: "Ngày chụp (từ metadata)",
      date_import: "Ngày nhập (thời gian chuyển)",
      vid_location: "Vị trí Video",
      vid_separate: "Thư mục Video riêng (/Video)",
      vid_mixed: "Trộn cùng ảnh (giống như trong máy)",
    }
  };

  function t(key, params = {}) {
    let str = translations[config.language || 'en'][key] || translations['en'][key] || key;
    for (const [pk, pv] of Object.entries(params)) {
      str = str.replace(`{${pk}}`, pv);
    }
    return str;
  }

  // ── DOM refs ───────────────────────────────────────────
  const navDashboard  = document.getElementById('nav-dashboard');
  const navHistory    = document.getElementById('nav-history');
  const navSettings   = document.getElementById('nav-settings');
  const viewContainer = document.getElementById('view-container');
  const pageTitle     = document.getElementById('page-title');
  const statusText    = document.getElementById('status-text');
  const statusPulse   = document.getElementById('status-pulse');
  const statusDot     = document.getElementById('status-dot');

  // ── Tauri Events ───────────────────────────────────────
  await listen('sd-card-detected', (event) => {
    detectedCard = event.payload;
    console.log('SD card detected:', detectedCard);
    if (currentState === 'idle' && config.autoIngest && config.destPath) {
      doIngest(detectedCard.drive_path);
    } else if (currentState === 'idle' && currentView === 'dashboard') {
      renderDashboard();
    }
    if (config.notifyTray) {
      showToast(`SD Card Detected: ${detectedCard.volume_label} (${detectedCard.file_count} files)`);
    }
  });

  await listen('copy-progress', (event) => {
    copyProgress = event.payload;
    if (currentView === 'dashboard' && currentState === 'copying') {
      updateProgressUI();
    }
  });

  async function toggleLanguage() {
    config.language = config.language === 'en' ? 'vi' : 'en';
    await invoke('save_config', { config }).catch(console.error);
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
      sidebar.outerHTML = getSidebarHTML();
      bindSidebarEvents(); // Re-bind since replaced
    }
    setView(currentView, t(currentView), `nav-${currentView}`);
  }

  function bindSidebarEvents() {
    const navDashboard  = document.getElementById('nav-dashboard');
    const navHistory    = document.getElementById('nav-history');
    const navSettings   = document.getElementById('nav-settings');
    const btnToggleLang = document.getElementById('btn-toggle-lang');

    if (navDashboard) navDashboard.addEventListener('click', () => setView('dashboard', t('dashboard'), 'nav-dashboard'));
    if (navHistory)   navHistory.addEventListener('click',   () => setView('history',   t('history'),   'nav-history'));
    if (navSettings)  navSettings.addEventListener('click',  () => setView('settings',  t('settings'),  'nav-settings'));
    if (btnToggleLang) btnToggleLang.addEventListener('click', toggleLanguage);
    
    if (window.lucide) window.lucide.createIcons();
  }

  await listen('copy-complete', (event) => {
    const result = event.payload;
    currentState = 'idle';
    
    // Add to history
    const session = {
      date: new Date().toLocaleString(),
      label: detectedCard ? detectedCard.volume_label : 'Manual Ingest',
      files: result.copied,
      skipped: result.skipped,
      photos: result.photos_copied,
      raw: result.raw_copied,
      videos: result.videos_copied,
      dest: result.destination,
      brands: result.brands,
      synced: config.syncRemote ? 'Synced ✓' : 'Local Only'
    };
    history.unshift(session);
    if (history.length > 50) history.pop();

    copyProgress = null;
    detectedCard = null;
    updateStatusUI();
    
    if (currentView === 'dashboard') renderDashboard();
    
    const toastMsg = `${t('done')} ${t('copied_skipped', { copied: result.copied, skipped: result.skipped })}<br><span style="font-size:11px;opacity:0.7">${result.destination}</span>`;
    showToast(toastMsg);
    if (config.playSound) {
      // Audio playback logic could go here if needed
    }
  });

  // ── Toast ──────────────────────────────────────────────
  function showToast(msg) {
    const container = document.getElementById('toast-container');
    if (!container) return;
    const t = document.createElement('div');
    t.className = 'toast-enter';
    t.style.cssText = 'background:#2a2a2a;border:1px solid #3d3d3d;padding:12px 20px;border-radius:12px;font-size:13.5px;color:#e5e5e5;pointer-events:auto;box-shadow:0 10px 30px rgba(0,0,0,0.5);display:flex;align-items:center;gap:10px;';
    t.innerHTML = `<i data-lucide="info" style="width:18px;height:18px;color:#14b8a6"></i> ${msg}`;
    container.appendChild(t);
    if (window.lucide) window.lucide.createIcons();
    setTimeout(() => { 
      t.style.opacity = '0'; 
      t.style.transform = 'translateY(10px)';
      t.style.transition = 'all 0.4s ease';
      setTimeout(() => t.remove(), 400); 
    }, 5000);
  }

  // ── Helpers ────────────────────────────────────────────
  function fmtETA(secs) {
    if (!secs || secs <= 0) return '—';
    if (secs > 3600) return `${Math.floor(secs/3600)}h ${Math.floor((secs%3600)/60)}m`;
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    const lang = config.language || 'en';
    if (lang === 'vi') return m > 0 ? `${m}p ${s}gi` : `${s}gi`;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  function getSidebarHTML() {
    const langLabel = config.language === 'en' ? 'Tiếng Việt' : 'English';
    return `
      <div id="sidebar" class="sidebar">
        <div class="flex flex-col gap-10">
          <!-- Logo -->
          <div class="flex items-center gap-3.5 px-2">
            <div class="w-9 h-9 rounded-xl bg-teal-500/10 flex items-center justify-center border border-teal-500/20 shadow-lg shadow-teal-500/5">
               <i data-lucide="camera" class="w-5 h-5 text-teal-500"></i>
            </div>
            <span class="text-[18px] font-bold tracking-tight text-white">FrameDrop</span>
          </div>
          
          <!-- Nav links -->
          <nav class="flex flex-col gap-1.5">
            <div id="nav-dashboard" class="nav-link">
              <i data-lucide="layout-dashboard"></i>
              <span>${t('dashboard')}</span>
            </div>
            <div id="nav-history" class="nav-link">
              <i data-lucide="clock"></i>
              <span>${t('history')}</span>
            </div>
          </nav>
        </div>

        <div class="flex flex-col gap-4">
          <!-- Language Toggle -->
          <button id="btn-toggle-lang" class="mx-1 flex items-center justify-center gap-2.5 px-4 py-2.5 rounded-xl border border-[#333] text-gray-500 hover:text-teal-400 hover:border-teal-500/30 transition-all font-mono text-[10px] font-bold uppercase tracking-widest bg-[#1a1a1a]/50">
            <i data-lucide="languages" class="w-4 h-4"></i>
            ${langLabel}
          </button>

          <div id="nav-settings" class="nav-link">
            <i data-lucide="settings"></i>
            <span>${t('settings')}</span>
          </div>
        </div>
      </div>
    `;
  }

  async function doIngest(drivePath) {
    if (!config.destPath) { 
      showToast(t('toast_dest_first')); 
      return; 
    }
    currentState = 'copying';
    copyProgress = null;
    updateStatusUI();
    renderDashboard();
    try {
      await invoke('start_manual_ingest', { drivePath, config });
    } catch(e) {
      showToast(t('toast_ingest_err') + e);
      currentState = 'idle';
      updateStatusUI();
      renderDashboard();
    }
  }

  // ── Dashboard Renderers ────────────────────────────────
  function getDashboardIdleHTML() {
    const hasCard = detectedCard !== null;
    const cardLabel = hasCard ? detectedCard.volume_label : '';
    const cardFiles = hasCard ? detectedCard.file_count : 0;
    const cardPath  = hasCard ? detectedCard.drive_path  : '';

    if (hasCard) {
      return `
        <div class="flex flex-col items-center justify-center h-[85%] w-full text-center gap-5 slide-in">
          <div class="w-28 h-28 rounded-[32px] bg-teal-500/10 flex items-center justify-center mb-2 border-2 border-teal-500/20 shadow-xl">
            <i data-lucide="usb" class="text-teal-400 w-12 h-12"></i>
          </div>
          <div class="flex flex-col gap-2">
            <h2 class="text-[22px] font-bold text-gray-100 m-0 tracking-tight">${t('sd_card_found')}<span class="text-teal-400">${cardLabel}</span></h2>
            <p class="text-[14px] text-gray-400 max-w-[420px] m-0 leading-relaxed mx-auto mt-1">
              ${t('ready_to_begin', { count: cardFiles, path: cardPath })}
            </p>
          </div>
          <div class="flex flex-col gap-3 mt-6 w-full max-w-[320px]">
            <button id="btn-auto-ingest" class="btn-primary w-full py-3 justify-center shadow-[0_10px_20px_rgba(20,184,166,0.2)]">
              <i data-lucide="download" class="w-5 h-5"></i>
              ${t('start_ingest')}
            </button>
            <button id="btn-manual-start" class="btn-secondary w-full py-3 justify-center">
              <i data-lucide="folder-search" class="w-5 h-5"></i>
              ${t('select_different')}
            </button>
          </div>
        </div>`;
    }

    return `
      <div class="flex flex-col items-center justify-center h-[85%] w-full text-center gap-5 slide-in">
        <div class="w-28 h-28 rounded-[32px] bg-[#222] border border-[#2d2d2d] flex items-center justify-center mb-2 shadow-lg">
          <i data-lucide="hard-drive" class="text-gray-600 w-12 h-12"></i>
        </div>
        <div class="flex flex-col gap-2">
          <h2 class="text-[22px] font-bold text-gray-100 m-0 tracking-tight">${t('watching_cards')}</h2>
          <p class="text-[14px] text-gray-400 max-w-[420px] m-0 leading-relaxed mx-auto mt-1">
            ${t('plug_in_card')}
          </p>
        </div>
        <div class="flex flex-col gap-3 mt-6 w-full max-w-[320px]">
          <button id="btn-manual-start" class="btn-primary w-full py-3 justify-center shadow-[0_10px_20px_rgba(20,184,166,0.1)]">
            <i data-lucide="plus" class="w-5 h-5"></i>
            ${t('manual_ingest')}
          </button>
          <button id="btn-open-dest" class="btn-secondary w-full py-3 justify-center">
            <i data-lucide="folder" class="w-5 h-5 text-gray-400"></i>
            ${t('open_destination')}
          </button>
        </div>
      </div>`;
  }

  function getDashboardCopyingHTML() {
    const p = copyProgress || { current: 0, total: 100, current_file: '...', speed_mbps: 0, eta_seconds: 0 };
    const pct = p.total > 0 ? Math.round((p.current / p.total) * 100) : 0;
    const label = detectedCard ? detectedCard.volume_label : 'Removable Drive';

    return `
      <div class="view-max-width slide-in flex flex-col gap-8">
        <div class="bg-[#222] border border-[#2d2d2d] rounded-2xl p-8 flex flex-col gap-6 shadow-2xl">
          <div class="flex justify-between items-start w-full">
            <div class="flex gap-5 items-center">
              <div class="w-14 h-14 rounded-2xl bg-teal-500/10 flex items-center justify-center border border-teal-500/20">
                <i data-lucide="refresh-cw" class="text-teal-400 w-7 h-7 status-active"></i>
              </div>
              <div class="flex flex-col">
                <h3 class="text-[18px] font-bold text-gray-100 m-0">${t('copying_from')}<span class="text-teal-400">${label}</span></h3>
                <span id="copy-counter" class="text-[14px] text-gray-500 mt-1">${t('files_counter', { current: p.current, total: p.total })}</span>
              </div>
            </div>
            <button id="btn-cancel" class="px-5 py-2.5 rounded-xl border border-red-900/30 bg-red-500/10 text-red-500 text-[13px] font-bold hover:bg-red-500/20 transition-all cursor-pointer">
              ${t('cancel')}
            </button>
          </div>
          
          <div class="flex flex-col gap-3 w-full">
            <div class="progress-container">
              <div id="progress-bar" class="progress-fill" style="width: ${pct}%;"></div>
            </div>
            <div class="flex justify-between items-center text-[13px] font-bold text-gray-400 px-1">
              <span id="progress-pct" class="text-teal-400">${pct}%</span>
              <div class="flex gap-4">
                <span id="progress-speed" class="flex items-center gap-1.5"><i data-lucide="zap" class="w-3.5 h-3.5 text-amber-500"></i> ${t('speed')}: ${p.speed_mbps.toFixed(1)} MB/s</span>
                <span id="progress-eta" class="flex items-center gap-1.5"><i data-lucide="clock" class="w-3.5 h-3.5 text-gray-500"></i> ${fmtETA(p.eta_seconds)}</span>
              </div>
            </div>
          </div>

          <div class="bg-[#1a1a1a] border border-[#2d2d2d] rounded-xl p-4 flex items-center gap-4 shadow-inner">
            <div class="w-10 h-10 rounded-lg bg-[#252525] flex items-center justify-center">
              <i data-lucide="file-type" class="w-5 h-5 text-gray-500"></i>
            </div>
            <div class="flex flex-col overflow-hidden">
              <span class="text-[11px] font-bold text-gray-600 uppercase tracking-widest">${t('current_file')}</span>
              <span id="current-file" class="text-[13.5px] text-gray-300 font-mono truncate">${p.current_file}</span>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-4 px-2">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-teal-500 status-active"></span>
            <h4 class="text-[11px] font-bold text-gray-500 tracking-widest m-0 uppercase">${t('recent_transfers')}</h4>
          </div>
          <div id="recent-files" class="flex flex-col gap-2"></div>
        </div>
      </div>`;
  }

  // Update numbers in the copying UI
  const recentFiles = [];
  function updateProgressUI() {
    if (!copyProgress) return;
    const p = copyProgress;
    const pct = p.total > 0 ? Math.round((p.current / p.total) * 100) : 0;

    const bar     = document.getElementById('progress-bar');
    const pctEl   = document.getElementById('progress-pct');
    const etaEl   = document.getElementById('progress-eta');
    const speedEl = document.getElementById('progress-speed');
    const fileEl  = document.getElementById('current-file');
    const counter = document.getElementById('copy-counter');
    const recent  = document.getElementById('recent-files');

    if (bar)     bar.style.width = pct + '%';
    if (pctEl)   pctEl.innerText = pct + '%';
    if (etaEl)   etaEl.innerHTML = `<i data-lucide="clock" class="w-3.5 h-3.5 text-gray-500"></i> ${fmtETA(p.eta_seconds)}`;
    if (speedEl) speedEl.innerHTML = `<i data-lucide="zap" class="w-3.5 h-3.5 text-amber-500"></i> ${p.speed_mbps.toFixed(1)} MB/s`;
    if (fileEl)  fileEl.innerText = p.current_file;
    if (counter) counter.innerText = `${p.current} of ${p.total} files`;

    if (p.current_file && (recentFiles.length === 0 || recentFiles[0] !== p.current_file)) {
      recentFiles.unshift(p.current_file);
      if (recentFiles.length > 6) recentFiles.pop();
    }
    if (recent) {
      recent.innerHTML = recentFiles.map((f, i) => {
        const opacity = Math.max(0.3, 1 - i * 0.15);
        return `<div class="flex items-center gap-3 text-[13px] font-mono text-gray-400 fade-in py-1" style="opacity:${opacity}"><i data-lucide="check" class="w-4 h-4 text-teal-500"></i>${f}</div>`;
      }).join('');
      if (window.lucide) window.lucide.createIcons();
    }
  }

  function renderDashboard() {
    viewContainer.innerHTML = currentState === 'copying' ? getDashboardCopyingHTML() : getDashboardIdleHTML();
    if (window.lucide) window.lucide.createIcons();
    bindDashboardButtons();
  }

  function bindDashboardButtons() {
    if (currentState === 'idle') {
      const btnAutoIngest = document.getElementById('btn-auto-ingest');
      if (btnAutoIngest && detectedCard) {
        btnAutoIngest.addEventListener('click', () => doIngest(detectedCard.drive_path));
      }

      const btnStart = document.getElementById('btn-manual-start');
      if (btnStart) {
        btnStart.addEventListener('click', async () => {
          try {
            const selected = await open({ directory: true, multiple: false, title: 'Select SD Card' });
            if (selected) {
              detectedCard = { drive_path: selected, volume_label: selected.split('\\').pop() || 'Folder', file_count: 0 };
              doIngest(selected);
            }
          } catch(e) { console.error(e); }
        });
      }

      const btnOpenDest = document.getElementById('btn-open-dest');
      if (btnOpenDest) {
        btnOpenDest.addEventListener('click', () => {
          if (config.destPath) invoke('open_folder', { path: config.destPath }).catch(console.error);
          else showToast('Set a destination path in Settings first.');
        });
      }
    } else {
      const btnCancel = document.getElementById('btn-cancel');
      if (btnCancel) {
        btnCancel.addEventListener('click', async () => {
          await invoke('cancel_ingest').catch(console.error);
          currentState = 'idle';
          copyProgress = null;
          updateStatusUI();
          renderDashboard();
        });
      }
    }
  }

  // ── Status pill ────────────────────────────────────────
  function updateStatusUI() {
    if (currentState === 'copying') {
      if (statusText)  statusText.innerText = t('copying');
      if (statusPulse) statusPulse.classList.add('pulse-status');
      if (statusDot)   statusDot.style.background = '#14b8a6';
    } else {
      if (statusText)  statusText.innerText = t('idle');
      if (statusPulse) statusPulse.classList.remove('pulse-status');
      if (statusDot)   statusDot.style.background = '#444';
    }
  }

  // ── History View ───────────────────────────────────────
  function getHistoryHTML() {
    if (history.length === 0) {
      return `
        <div class="view-max-width slide-in flex flex-col items-center justify-center h-[70%] text-center gap-4">
          <div class="w-20 h-20 rounded-full bg-[#222] flex items-center justify-center border border-[#2d2d2d]">
            <i data-lucide="history" class="text-gray-700 w-8 h-8"></i>
          </div>
          <p class="text-gray-500 text-[14px]">${t('no_history')}</p>
        </div>`;
    }

    const items = history.map(h => {
      let brandList = '';
      if (h.brands) {
        brandList = Object.entries(h.brands)
          .map(([b, count]) => `<span class="bg-[#2d2d2d] border border-[#3d3d3d] px-2 py-0.5 rounded text-[11px] font-bold text-gray-300">${b}: ${count}</span>`)
          .join(' ');
      }

      return `
        <div class="history-item flex flex-col gap-4 slide-in">
          <div class="flex justify-between items-start">
            <div class="flex gap-4 items-center">
              <div class="w-10 h-10 rounded-lg bg-teal-500/10 flex items-center justify-center border border-teal-500/10">
                <i data-lucide="folder-check" class="text-teal-500 w-5 h-5"></i>
              </div>
              <div class="flex flex-col">
                <span class="text-[15px] font-bold text-gray-100">${h.label}</span>
                <span class="text-[12px] text-gray-500">${h.date}</span>
              </div>
            </div>
            <div class="flex flex-col items-end gap-1">
              <div class="flex gap-2 items-center text-[12px] font-bold">
                <span class="text-teal-400">${h.raw || 0} ${t('raw')}</span>
                <span class="text-gray-500">•</span>
                <span class="text-teal-400">${h.photos || 0} ${t('photos')}</span>
                <span class="text-gray-500">•</span>
                <span class="text-teal-400">${h.videos || 0} ${t('videos')}</span>
              </div>
              <span class="text-[11px] text-gray-600">${h.skipped || 0} ${t('duplicates')} • ${h.synced}</span>
            </div>
          </div>
          <div class="flex flex-wrap gap-2">${brandList}</div>
          <div class="flex items-center justify-between border-t border-[#2d2d2d] pt-3 mt-1">
             <span class="text-[11px] text-gray-500 font-mono truncate max-w-[80%]">${h.dest}</span>
             <button class="text-[11px] text-teal-500 hover:underline font-bold" onclick="window.__TAURI__.core.invoke('open_folder', { path: '${h.dest.replace(/\\/g, '\\\\')}' })">${t('open_folder')}</button>
          </div>
        </div>`;
    }).join('');

    return `
      <div class="view-max-width slide-in flex flex-col gap-6">
        <div class="flex justify-between items-center px-1">
          <h2 class="text-[20px] font-bold text-gray-100 m-0">${t('recent_ingests')}</h2>
          <button id="btn-clear-history" class="btn-secondary py-1.5 px-3 text-[12px]">${t('clear_history')}</button>
        </div>
        <div class="flex flex-col gap-4">${items}</div>
      </div>`;
  }

  // ── Settings View ──────────────────────────────────────
  function getSettingsHTML() {
    const dp = config.destPath || t('toast_dest_first');
    
    const dateActive    = config.organizeDate    ? 'active' : '';
    const brandActive   = config.organizeBrand   ? 'active' : '';
    const rawActive     = config.includeRaw      ? 'active' : '';
    const jpgActive     = config.includeJpeg     ? 'active' : '';
    const vidActive     = config.includeVideo    ? 'active' : '';
    const trayActive    = config.notifyTray      ? 'active' : '';
    const soundActive   = config.playSound       ? 'active' : '';
    const startActive   = config.startWithOs     ? 'active' : '';
    const minActive     = config.minimizeToTray  ? 'active' : '';
    const autoActive    = config.autoIngest      ? 'active' : '';
    const syncActive    = config.syncRemote      ? 'active' : '';
    const webhookActive = config.webhookEnabled   ? 'active' : '';

    let previewParts = [config.destPath || '/Photos'];
    if (config.organizeDate)  previewParts.push('2026-03-30');
    if (config.organizeBrand) previewParts.push('ILCE-7RM4');
    previewParts.push('DSC00001.ARW');
    const previewStr = previewParts.join(' <span class="text-gray-600 mx-1">/</span> ');

    return `
      <div class="view-max-width slide-in settings-grid">
        
        <!-- Destination -->
        <div class="section-card">
          <label class="section-title">${t('dest_path')}</label>
          <div class="flex flex-col gap-4">
             <div class="flex gap-3 items-center">
              <div class="flex-1 bg-[#1a1a1a] border border-[#333] rounded-xl p-3.5 flex items-center gap-3.5 shadow-inner overflow-hidden">
                <i data-lucide="folder" class="w-5 h-5 text-teal-500/80"></i>
                <span id="setting-dest-path" class="text-[14px] text-gray-300 font-mono flex-1 truncate">${dp}</span>
              </div>
              <button id="btn-change-dest" class="btn-secondary h-[52px] px-6 text-[14px] font-bold whitespace-nowrap rounded-xl">${t('browse')}</button>
            </div>
            <div class="flex gap-2">
              <button id="btn-open-explorer" class="btn-outline px-4 py-2 rounded-lg">
                <i data-lucide="external-link" class="w-3.5 h-3.5"></i>
                ${t('open_explorer')}
              </button>
            </div>
          </div>
        </div>

        <!-- Folder Structure -->
        <div class="section-card">
          <label class="section-title">${t('folder_structure')}</label>
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-date">
              <div class="flex flex-col">
                <span class="text-[14px] font-medium text-gray-200">${t('org_date')}</span>
                <span class="text-[12px] text-gray-500">${t('org_date_desc')}</span>
              </div>
              <div id="toggle-date" class="toggle-track ${dateActive}"><div class="toggle-thumb"></div></div>
            </div>
            
            <div id="date-source-settings" class="flex flex-col gap-3 pl-2 mt-1 pb-2 border-l-2 border-[#2d2d2d] ml-1.5 ${config.organizeDate ? '' : 'hidden'}">
              <label class="text-[11px] text-gray-500 font-bold uppercase tracking-wider mb-1">${t('date_source')}</label>
              <div class="flex flex-col gap-2.5">
                <div class="flex items-center gap-3 cursor-pointer group" id="opt-date-capture">
                  <div class="w-5 h-5 rounded-full border-2 border-[#444] flex items-center justify-center transition-all ${config.dateSource === 'capture' ? 'border-teal-500' : 'group-hover:border-gray-500'}">
                    <div class="w-2.5 h-2.5 rounded-full bg-teal-500 transition-all ${config.dateSource === 'capture' ? 'scale-100 opacity-100' : 'scale-0 opacity-0'}"></div>
                  </div>
                  <span class="text-[13px] ${config.dateSource === 'capture' ? 'text-gray-200 font-medium' : 'text-gray-400 group-hover:text-gray-300'}">${t('date_capture')}</span>
                </div>
                <div class="flex items-center gap-3 cursor-pointer group" id="opt-date-import">
                  <div class="w-5 h-5 rounded-full border-2 border-[#444] flex items-center justify-center transition-all ${config.dateSource === 'import' ? 'border-teal-500' : 'group-hover:border-gray-500'}">
                    <div class="w-2.5 h-2.5 rounded-full bg-teal-500 transition-all ${config.dateSource === 'import' ? 'scale-100 opacity-100' : 'scale-0 opacity-0'}"></div>
                  </div>
                  <span class="text-[13px] ${config.dateSource === 'import' ? 'text-gray-200 font-medium' : 'text-gray-400 group-hover:text-gray-300'}">${t('date_import')}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-camera">
              <div class="flex flex-col">
                <span class="text-[14px] font-medium text-gray-200">${t('org_model')}</span>
                <span class="text-[12px] text-gray-500">${t('org_model_desc')}</span>
              </div>
              <div id="toggle-camera" class="toggle-track ${brandActive}"><div class="toggle-thumb"></div></div>
            </div>
            <div class="mt-4 bg-[#1a1a1a] border border-[#333] rounded-xl p-4 flex flex-col gap-2 relative shadow-inner">
              <span class="text-[9px] font-bold text-teal-500/30 uppercase tracking-widest absolute top-4 right-5">${t('live_preview')}</span>
              <div id="preview-path" class="text-[14px] text-teal-400 font-mono flex items-center gap-3 mt-1 truncate">
                <i data-lucide="folder-tree" class="w-4 h-4 text-gray-700"></i>
                <div class="flex items-center leading-none">${previewStr}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Video Management -->
        <div class="section-card">
          <label class="section-title">${t('vid_files')}</label>
          <div class="flex flex-col gap-4">
            <div id="video-folder-settings" class="flex flex-col gap-3">
              <label class="text-[11px] text-gray-500 font-bold uppercase tracking-wider mb-1">${t('vid_location')}</label>
              <div class="flex flex-col gap-2.5">
                <div class="flex items-center gap-3 cursor-pointer group" id="opt-vid-separate">
                  <div class="w-5 h-5 rounded-full border-2 border-[#444] flex items-center justify-center transition-all ${config.videoFolder === 'separate' ? 'border-teal-500' : 'group-hover:border-gray-500'}">
                    <div class="w-2.5 h-2.5 rounded-full bg-teal-500 transition-all ${config.videoFolder === 'separate' ? 'scale-100 opacity-100' : 'scale-0 opacity-0'}"></div>
                  </div>
                  <span class="text-[13px] ${config.videoFolder === 'separate' ? 'text-gray-200 font-medium' : 'text-gray-400 group-hover:text-gray-300'}">${t('vid_separate')}</span>
                </div>
                <div class="flex items-center gap-3 cursor-pointer group" id="opt-vid-mixed">
                  <div class="w-5 h-5 rounded-full border-2 border-[#444] flex items-center justify-center transition-all ${config.videoFolder === 'mixed' ? 'border-teal-500' : 'group-hover:border-gray-500'}">
                    <div class="w-2.5 h-2.5 rounded-full bg-teal-500 transition-all ${config.videoFolder === 'mixed' ? 'scale-100 opacity-100' : 'scale-0 opacity-0'}"></div>
                  </div>
                  <span class="text-[13px] ${config.videoFolder === 'mixed' ? 'text-gray-200 font-medium' : 'text-gray-400 group-hover:text-gray-300'}">${t('vid_mixed')}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-scan-video">
              <div class="flex flex-col">
                <span class="text-[14px] font-medium text-gray-200">${t('scan_video_dirs')}</span>
                <span class="text-[12px] text-gray-500">${t('scan_video_dirs_desc')}</span>
              </div>
              <div id="toggle-scan-video" class="toggle-track ${config.scanVideoDirs ? 'active' : ''}"><div class="toggle-thumb"></div></div>
            </div>
          </div>
        </div>

        <!-- Remote Sync -->
        <div class="section-card">
          <label class="section-title">${t('remote_sync')}</label>
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-sync">
              <div class="flex flex-col">
                <span class="text-[14px] font-medium text-gray-200">${t('sync_desc')}</span>
                <span class="text-[12px] text-gray-500">${t('auto_promptless')}</span>
              </div>
              <div id="toggle-sync" class="toggle-track ${syncActive}"><div class="toggle-thumb"></div></div>
            </div>
            <div id="sync-settings" class="flex flex-col gap-3 mt-1 ${config.syncRemote ? '' : 'opacity-40 pointer-events-none transition-opacity'}">
              <div class="grid grid-cols-2 gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="text-[11px] text-gray-500 font-bold uppercase">${t('remote_ip')}</label>
                  <input type="text" id="input-remote-ip" class="input-field" value="${config.remoteIp}" placeholder="192.168.1.50">
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="text-[11px] text-gray-500 font-bold uppercase">${t('trans_method')}</label>
                  <select id="select-remote-method" class="select-field">
                    <option value="SMB" ${config.remoteMethod === 'SMB' ? 'selected' : ''}>SMB Share</option>
                    <option value="SFTP" ${config.remoteMethod === 'SFTP' ? 'selected' : ''}>SSH + SFTP</option>
                  </select>
                </div>
              </div>
              <div class="flex flex-col gap-1.5">
                <label class="text-[11px] text-gray-500 font-bold uppercase">${t('remote_dest')}</label>
                <input type="text" id="input-remote-path" class="input-field" value="${config.remotePath}" placeholder="/Volumes/Photos/Ingests">
              </div>
              <button id="btn-test-connection" class="btn-secondary w-fit py-1.5 px-4 text-[12px] mt-1">${t('test_conn')}</button>
              <div id="test-result" class="text-[11px] font-medium hidden"></div>
            </div>
          </div>
        </div>

        <!-- Webhook Notification -->
        <div class="section-card">
          <label class="section-title">${t('webhook_title')}</label>
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-webhook">
              <div class="flex flex-col">
                <span class="text-[14px] font-medium text-gray-200">${t('enable_webhook')}</span>
                <span class="text-[12px] text-gray-500">${t('webhook_desc')}</span>
              </div>
              <div id="toggle-webhook" class="toggle-track ${webhookActive}"><div class="toggle-thumb"></div></div>
            </div>
            <div id="webhook-settings" class="flex flex-col gap-3 mt-1 ${config.webhookEnabled ? '' : 'opacity-40 pointer-events-none transition-opacity'}">
              <div class="flex flex-col gap-1.5">
                <label class="text-[11px] text-gray-500 font-bold uppercase">${t('webhook_url')}</label>
                <div class="flex gap-2">
                  <input type="text" id="input-webhook-url" class="input-field" value="${config.webhookUrl}" placeholder="https://discord.com/api/webhooks/...">
                  <button id="btn-test-webhook" class="btn-secondary py-1.5 px-3.5 text-[13px] whitespace-nowrap">${t('test')}</button>
                </div>
              </div>
              <div class="flex flex-col gap-1.5 mt-1">
                <label class="text-[11px] text-gray-500 font-bold uppercase">${t('ping_id')}</label>
                <input type="text" id="input-webhook-ping-id" class="input-field" value="${config.webhookPingId}" placeholder="Discord User ID">
              </div>
              <div id="webhook-test-result" class="text-[11px] font-medium hidden"></div>
            </div>
          </div>
        </div>

        <!-- File Types -->
        <div class="section-card">
          <label class="section-title">${t('file_types')}</label>
          <div class="flex flex-col gap-3">
            <div class="checkbox-container ${rawActive}" id="check-raw">
              <div class="checkbox-box"><i data-lucide="check"></i></div>
              <span>${t('raw_files')}</span>
            </div>
            <div class="checkbox-container ${jpgActive}" id="check-jpg">
              <div class="checkbox-box"><i data-lucide="check"></i></div>
              <span>${t('jpg_files')}</span>
            </div>
            <div class="checkbox-container ${vidActive}" id="check-vid">
              <div class="checkbox-box"><i data-lucide="check"></i></div>
              <span>${t('vid_files')}</span>
            </div>
          </div>
        </div>

        <!-- Notifications & Behavior -->
        <div class="grid grid-cols-2 gap-5">
          <div class="section-card">
            <label class="section-title">${t('notifs')}</label>
            <div class="flex flex-col gap-4">
              <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-tray-notif">
                <span class="text-[14px] text-gray-200">${t('tray_notif')}</span>
                <div id="toggle-tray-notif" class="toggle-track ${trayActive}"><div class="toggle-thumb"></div></div>
              </div>
              <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-sound">
                <span class="text-[14px] text-gray-200">${t('sound_complete')}</span>
                <div id="toggle-sound" class="toggle-track ${soundActive}"><div class="toggle-thumb"></div></div>
              </div>
            </div>
          </div>
          <div class="section-card">
            <label class="section-title">${t('app_behavior')}</label>
            <div class="flex flex-col gap-4">
              <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-start">
                <span class="text-[14px] text-gray-200">${t('start_win')}</span>
                <div id="toggle-start" class="toggle-track ${startActive}"><div class="toggle-thumb"></div></div>
              </div>
              <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-minimize">
                <span class="text-[14px] text-gray-200">${t('min_close')}</span>
                <div id="toggle-minimize" class="toggle-track ${minActive}"><div class="toggle-thumb"></div></div>
              </div>
              <div class="flex items-center justify-between cursor-pointer group" id="row-toggle-auto">
                <span class="text-[14px] text-gray-200">${t('auto_promptless')}</span>
                <div id="toggle-auto" class="toggle-track ${autoActive}"><div class="toggle-thumb"></div></div>
              </div>
            </div>
          </div>
        </div>
      </div>`;
  }

  // ── View Router ────────────────────────────────────────
  function setView(viewName, title, navId) {
    currentView = viewName;

    if (viewName === 'dashboard') {
      renderDashboard();
    } else if (viewName === 'history') {
      viewContainer.innerHTML = getHistoryHTML();
      if (window.lucide) window.lucide.createIcons();
      bindHistoryEvents();
    } else if (viewName === 'settings') {
      viewContainer.innerHTML = getSettingsHTML();
      if (window.lucide) window.lucide.createIcons();
      bindSettingsEvents();
    }

    if (pageTitle) pageTitle.innerText = title;
    updateStatusUI();

    [navDashboard, navHistory, navSettings].forEach(nav => {
      if (nav) { nav.classList.remove('active-nav'); nav.style.background = 'none'; nav.style.color = '#888'; }
    });
    const activeNav = document.getElementById(navId);
    if (activeNav) activeNav.classList.add('active-nav');
  }

  const sidebarEl = document.getElementById('sidebar');
  if (sidebarEl) {
    sidebarEl.outerHTML = getSidebarHTML();
    bindSidebarEvents();
  }

  // ── Events Binding ─────────────────────────────────────
  function bindHistoryEvents() {
    const btnClear = document.getElementById('btn-clear-history');
    if (btnClear) btnClear.addEventListener('click', () => { 
      history = []; 
      setView('history', t('history'), 'nav-history'); 
    });
  }

  function bindSettingsEvents() {
    const ids = [
      ['toggle-date', 'organizeDate', 'row-toggle-date'],
      ['toggle-camera', 'organizeBrand', 'row-toggle-camera'],
      ['toggle-sync', 'syncRemote', 'row-toggle-sync'],
      ['toggle-tray-notif', 'notifyTray', 'row-toggle-tray-notif'],
      ['toggle-sound', 'playSound', 'row-toggle-sound'],
      ['toggle-start', 'startWithOs', 'row-toggle-start'],
      ['toggle-minimize', 'minimizeToTray', 'row-toggle-minimize'],
      ['toggle-auto', 'autoIngest', 'row-toggle-auto'],
      ['toggle-webhook', 'webhookEnabled', 'row-toggle-webhook'],
      ['toggle-scan-video', 'scanVideoDirs', 'row-toggle-scan-video'],
      ['check-raw', 'includeRaw'],
      ['check-jpg', 'includeJpeg'],
      ['check-vid', 'includeVideo']
    ];

    function updatePreview() {
      const previewTitle = document.getElementById('preview-path');
      if (!previewTitle) return;
      let parts = [config.destPath || '/Photos'];
      if (config.organizeDate) {
        if (config.dateSource === 'import') {
          const today = new Date().toISOString().split('T')[0];
          parts.push(today);
        } else {
          parts.push('2026-03-30');
        }
      }
      if (config.organizeBrand) parts.push('ILCE-7RM4');
      
      // If separate video is on, show a video example in the preview
      if (config.videoFolder === 'separate') {
        parts.push('<span class="text-teal-500/50">Video</span>');
      }
      
      parts.push('DSC00001.ARW');
      previewTitle.innerHTML = `<i data-lucide="folder-tree" class="w-3.5 h-3.5 text-gray-600"></i> ${parts.join(' <span class="text-gray-600 mx-1">/</span> ')}`;
      if (window.lucide) window.lucide.createIcons();
    }

    async function save() {
      await invoke('save_config', { config }).catch(console.error);
    }

    ids.forEach(([id, key, rowId]) => {
      const el = document.getElementById(rowId || id);
      if (el) {
        el.addEventListener('click', () => {
          const track = id.startsWith('check') ? el : document.getElementById(id);
          track.classList.toggle('active');
          config[key] = track.classList.contains('active');
          
          if (key === 'organizeDate') {
            const dateSettings = document.getElementById('date-source-settings');
            if (dateSettings) dateSettings.classList.toggle('hidden', !config[key]);
          }
          if (key === 'syncRemote') {
            const settings = document.getElementById('sync-settings');
            if (settings) {
              settings.classList.toggle('opacity-40', !config[key]);
              settings.classList.toggle('pointer-events-none', !config[key]);
            }
          }
          if (key === 'webhookEnabled') {
            const settings = document.getElementById('webhook-settings');
            if (settings) {
              settings.classList.toggle('opacity-40', !config[key]);
              settings.classList.toggle('pointer-events-none', !config[key]);
            }
          }
          updatePreview();
          save();
        });
      }
    });

    // Date source radios
    ['opt-date-capture', 'opt-date-import'].forEach(id => {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener('click', () => {
          config.dateSource = id.includes('capture') ? 'capture' : 'import';
          setView('settings', t('settings'), 'nav-settings');
          updatePreview();
          save();
        });
      }
    });

    // Video folder radios
    ['opt-vid-separate', 'opt-vid-mixed'].forEach(id => {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener('click', () => {
          config.videoFolder = id.includes('separate') ? 'separate' : 'mixed';
          setView('settings', t('settings'), 'nav-settings');
          updatePreview();
          save();
        });
      }
    });

    // Inputs
    ['input-remote-ip', 'input-remote-path', 'select-remote-method', 'input-webhook-url', 'input-webhook-ping-id'].forEach(id => {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener('change', () => {
          if (id === 'input-webhook-url') {
             config.webhookUrl = el.value;
          } else if (id === 'input-webhook-ping-id') {
             config.webhookPingId = el.value;
          } else {
            const key = id.replace('input-', '').replace('select-', '').replace('remote-', 'remote');
            const camelKey = key.split('-').map((s,i)=>i===0?s:s[0].toUpperCase()+s.slice(1)).join('');
            config[camelKey === 'remoteIp' ? 'remoteIp' : camelKey === 'remotePath' ? 'remotePath' : 'remoteMethod'] = el.value;
          }
          save();
        });
      }
    });

    const btnTestWebhook = document.getElementById('btn-test-webhook');
    if (btnTestWebhook) {
      btnTestWebhook.addEventListener('click', async () => {
        const resEl = document.getElementById('webhook-test-result');
        if (resEl) {
          resEl.innerText = t('copying');
          resEl.classList.remove('hidden', 'text-red-500', 'text-teal-500');
          try {
            const res = await invoke('test_webhook', { 
              url: config.webhookUrl, 
              pingId: config.webhookPingId,
              language: config.language
            });
            resEl.innerText = 'Success: ' + res;
            resEl.classList.add('text-teal-500');
          } catch(e) {
            resEl.innerText = 'Error: ' + e;
            resEl.classList.add('text-red-500');
          }
        }
      });
    }

    const btnOpenExp = document.getElementById('btn-open-explorer');
    if (btnOpenExp) {
      btnOpenExp.addEventListener('click', () => {
        if (config.destPath) invoke('open_folder', { path: config.destPath }).catch(console.error);
        else showToast('Destination path is not set.');
      });
    }

    const btnTest = document.getElementById('btn-test-connection');
    if (btnTest) {
      btnTest.addEventListener('click', async () => {
        const resEl = document.getElementById('test-result');
        if (resEl) {
          resEl.innerText = 'Testing...';
          resEl.classList.remove('hidden', 'text-red-500', 'text-teal-500');
          try {
            const res = await invoke('test_remote_connection', { host: config.remoteIp, path: config.remotePath });
            resEl.innerText = 'Success: ' + res;
            resEl.classList.add('text-teal-500');
          } catch(e) {
            resEl.innerText = 'Error: ' + e;
            resEl.classList.add('text-red-500');
          }
        }
      });
    }

    const btnChange = document.getElementById('btn-change-dest');
    if (btnChange) {
      btnChange.addEventListener('click', async () => {
        const selected = await open({ directory: true, multiple: false });
        if (selected) {
          config.destPath = selected;
          const el = document.getElementById('setting-dest-path');
          if (el) el.innerText = selected;
          updatePreview();
          save();
        }
      });
    }
  }

  // ── Window Controls ───────────────────────────────────
  function bindTitleBarEvents() {
    // Current window instance from Tauri Global API
    const appWindow = window.__TAURI__.window.getCurrentWindow();
    
    document.getElementById('titlebar-minimize')?.addEventListener('click', () => appWindow.minimize());
    document.getElementById('titlebar-maximize')?.addEventListener('click', () => appWindow.toggleMaximize());
    document.getElementById('titlebar-close')?.addEventListener('click', () => appWindow.close());
  }

  // ── Init ───────────────────────────────────────────────
  bindTitleBarEvents();
  setView('dashboard', t('dashboard'), 'nav-dashboard');
  updateStatusUI();
});