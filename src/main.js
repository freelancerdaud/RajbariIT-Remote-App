// ============================================
// RajbariIT Remote Lite v1.0 — Sidebar Navigation
// ============================================

const { invoke } = window.__TAURI__.core;

// ============ State ============
let isHosting = false;
let isConnected = false;
let discoveredDevices = [];
let scanInterval = null;
let currentPage = 'dashboard';

// ============ DOM Ready ============
document.addEventListener('DOMContentLoaded', async () => {
    console.log('RajbariIT Remote starting...');

    // Initialize sidebar navigation
    setupNavigation();

    // Set device name
    try {
        const hostname = await invoke('get_device_name');
        setText('device-name', hostname || 'My Device');
        setText('sidebar-device-name', hostname || 'My Device');
    } catch (e) {
        setText('device-name', 'My Device');
        setText('sidebar-device-name', 'My Device');
    }

    // Generate initial PIN
    await refreshPin();

    // Start broadcasting
    try {
        const name = getText('device-name');
        await invoke('start_broadcasting', { deviceName: name });
    } catch (e) {
        console.warn('Broadcast error:', e);
    }

    // Start scanning
    startDeviceScan();

    // Start file receive server
    try {
        await invoke('start_receive_server', { port: 9096, saveDir: '' });
    } catch (e) {
        console.warn('Receive server error:', e);
    }

    // Setup all event listeners
    setupEventListeners();
});

// ============ Navigation ============
function setupNavigation() {
    const menuItems = document.querySelectorAll('.menu-item');
    menuItems.forEach(item => {
        item.addEventListener('click', () => {
            const page = item.dataset.page;
            if (page === currentPage) return;
            navigateTo(page);
        });
    });
}

function navigateTo(pageId) {
    // Update menu
    document.querySelectorAll('.menu-item').forEach(item => {
        item.classList.toggle('active', item.dataset.page === pageId);
    });

    // Update pages
    document.querySelectorAll('.page').forEach(page => {
        page.classList.toggle('active', page.id === `page-${pageId}`);
    });

    currentPage = pageId;
}

// ============ PIN Management ============
async function refreshPin() {
    try {
        const pin = await invoke('generate_pin');
        setText('security-pin', pin);
        setText('sidebar-pin', pin);
        setText('pin-large', pin);
    } catch (e) {
        console.warn('PIN error:', e);
    }
}

// ============ Device Discovery ============
async function scanDevices() {
    try {
        const devices = await invoke('discover_devices');
        discoveredDevices = devices;
        renderDeviceList(devices);

        // Update badge & stats
        const count = devices.length;
        setText('device-count', count.toString());
        setText('stat-devices', count.toString());
    } catch (e) {
        console.warn('Scan error:', e);
    }
}

function startDeviceScan() {
    scanDevices();
    scanInterval = setInterval(scanDevices, 5000);
}

function renderDeviceList(devices) {
    const list = document.getElementById('devices-list');
    const empty = document.getElementById('devices-empty');

    // Remove old device rows
    list.querySelectorAll('.device-row').forEach(r => r.remove());

    if (!devices || devices.length === 0) {
        empty.style.display = 'flex';
        return;
    }

    empty.style.display = 'none';

    devices.forEach(device => {
        const osIcon = getOsIcon(device.os);
        const row = document.createElement('div');
        row.className = 'device-row';
        row.innerHTML = `
            <div class="device-os-icon">${osIcon}</div>
            <div class="device-info">
                <div class="device-name">${esc(device.name)}</div>
                <div class="device-ip">${esc(device.ip)}:${device.port}</div>
            </div>
            <div class="device-actions">
                <button class="btn-device btn-device-connect" onclick="quickConnect('${esc(device.ip)}')">Connect</button>
                <button class="btn-device" onclick="sendFileTo('${esc(device.ip)}')">📁 Files</button>
            </div>
        `;
        list.appendChild(row);
    });
}

function getOsIcon(os) {
    const l = (os || '').toLowerCase();
    if (l.includes('win')) return '🪟';
    if (l.includes('mac') || l.includes('darwin')) return '🍎';
    if (l.includes('android')) return '🤖';
    if (l.includes('ios') || l.includes('iphone')) return '📱';
    if (l.includes('linux')) return '🐧';
    return '💻';
}

// ============ Screen Hosting ============
async function startHosting() {
    try {
        await invoke('start_screen_host', { port: 9095 });
        isHosting = true;
        hide('btn-start-host'); show('btn-stop-host');
        hide('btn-start-screen'); show('btn-stop-screen');
        document.getElementById('host-status').innerHTML = '<span class="status-dot status-dot-green"></span> Hosting';
        document.getElementById('screen-host-status').innerHTML = '<span class="status-dot status-dot-green"></span> Hosting on :9095';
        document.getElementById('connection-status').textContent = '● Hosting';
        document.getElementById('connection-status').className = 'status-badge status-connected';
    } catch (e) {
        alert('Failed to start hosting: ' + e);
    }
}

async function stopHosting() {
    try {
        await invoke('stop_screen_host');
        isHosting = false;
        show('btn-start-host'); hide('btn-stop-host');
        show('btn-start-screen'); hide('btn-stop-screen');
        document.getElementById('host-status').innerHTML = '<span class="status-dot status-dot-red"></span> Stopped';
        document.getElementById('screen-host-status').innerHTML = '<span class="status-dot status-dot-red"></span> Not hosting';
        document.getElementById('connection-status').textContent = '● Ready';
        document.getElementById('connection-status').className = 'status-badge status-ready';
    } catch (e) {
        console.warn('Stop hosting error:', e);
    }
}

// ============ Remote Connection ============
async function connectToTarget() {
    const ip = val('target-ip');
    const pin = val('target-pin');

    if (!ip) return alert('Please enter the target IP address');
    if (!pin || pin.length !== 6) return alert('Please enter a valid 6-digit PIN');

    try {
        const valid = await invoke('validate_pin', { pin });
        if (!valid) return alert('Invalid PIN');

        await invoke('connect_to_host', { ip, port: 9095 });
        isConnected = true;

        setText('session-target', ip);
        setText('control-status-text', 'Connected to ' + ip);
        document.getElementById('control-icon').textContent = '✅';
        show('canvas-wrapper');
        document.getElementById('connection-status').textContent = '● Connected';
        document.getElementById('connection-status').className = 'status-badge status-connected';
        setText('stat-connections', '1');

        startFrameReceiver();
        navigateTo('control');
    } catch (e) {
        alert('Connection failed: ' + e);
    }
}

function quickConnect(ip) {
    document.getElementById('target-ip').value = ip;
    navigateTo('dashboard');
    document.getElementById('target-pin').focus();
}

async function disconnect() {
    isConnected = false;
    hide('canvas-wrapper');
    document.getElementById('control-icon').textContent = '🔌';
    setText('control-status-text', 'Not Connected');
    document.getElementById('connection-status').textContent = '● Ready';
    document.getElementById('connection-status').className = 'status-badge status-ready';
    setText('stat-connections', '0');
}

// ============ Frame Receiver ============
async function startFrameReceiver() {
    const canvas = document.getElementById('remote-screen');
    const ctx = canvas.getContext('2d');

    const loop = async () => {
        if (!isConnected) return;
        try {
            const b64 = await invoke('capture_frame');
            if (b64) {
                const img = new Image();
                img.onload = () => {
                    canvas.width = img.width;
                    canvas.height = img.height;
                    ctx.drawImage(img, 0, 0);
                };
                img.src = 'data:image/jpeg;base64,' + b64;
            }
        } catch (_) {}
        if (isConnected) requestAnimationFrame(loop);
    };
    loop();
}

// ============ Input Forwarding ============
function setupRemoteInput() {
    const canvas = document.getElementById('remote-screen');
    canvas.addEventListener('mousemove', async (e) => {
        if (!isConnected) return;
        const r = canvas.getBoundingClientRect();
        const x = Math.round((e.clientX - r.left) / r.width * 65535);
        const y = Math.round((e.clientY - r.top) / r.height * 65535);
        try { await invoke('send_mouse_event', { x, y, action: 'move' }); } catch (_) {}
    });
    canvas.addEventListener('mousedown', async (e) => {
        if (!isConnected) return;
        const action = e.button === 2 ? 'right_click' : 'left_click';
        try { await invoke('send_mouse_event', { x: 0, y: 0, action }); } catch (_) {}
    });
    canvas.addEventListener('keydown', async (e) => {
        if (!isConnected) return;
        e.preventDefault();
        try {
            await invoke('send_key_event', {
                key: e.key,
                modifiers: { ctrl: e.ctrlKey, alt: e.altKey, shift: e.shiftKey, meta: e.metaKey }
            });
        } catch (_) {}
    });
    canvas.addEventListener('contextmenu', e => e.preventDefault());
}

// ============ File Transfer ============
function setupDragDrop() {
    const zone = document.getElementById('drop-zone');
    zone.addEventListener('dragover', e => { e.preventDefault(); zone.classList.add('drag-over'); });
    zone.addEventListener('dragleave', () => zone.classList.remove('drag-over'));
    zone.addEventListener('drop', e => { e.preventDefault(); zone.classList.remove('drag-over'); });
    zone.addEventListener('click', () => document.getElementById('file-input').click());
}

async function sendFileTo(ip) {
    document.getElementById('transfer-ip').value = ip;
    navigateTo('transfer');
}

// ============ Event Listeners ============
function setupEventListeners() {
    // Dashboard
    on('btn-refresh-pin', 'click', refreshPin);
    on('btn-start-host', 'click', startHosting);
    on('btn-stop-host', 'click', stopHosting);
    on('btn-connect', 'click', connectToTarget);

    // Discovery
    on('btn-scan', 'click', scanDevices);

    // Screen
    on('btn-start-screen', 'click', startHosting);
    on('btn-stop-screen', 'click', stopHosting);

    // Control
    on('btn-disconnect', 'click', disconnect);

    // Security
    on('btn-regen-pin', 'click', refreshPin);
    on('btn-copy-pin', 'click', () => {
        const pin = getText('pin-large');
        navigator.clipboard.writeText(pin).catch(() => {});
    });

    // File Transfer
    setupDragDrop();
    setupRemoteInput();
}

// ============ Utilities ============
function setText(id, text) { const el = document.getElementById(id); if (el) el.textContent = text; }
function getText(id) { const el = document.getElementById(id); return el ? el.textContent : ''; }
function val(id) { const el = document.getElementById(id); return el ? el.value.trim() : ''; }
function show(id) { const el = document.getElementById(id); if (el) el.style.display = 'flex'; }
function hide(id) { const el = document.getElementById(id); if (el) el.style.display = 'none'; }
function on(id, evt, fn) { const el = document.getElementById(id); if (el) el.addEventListener(evt, fn); }
function esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
