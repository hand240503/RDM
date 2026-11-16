(() => {
  'use strict';
  const API = '/api/v1';
  const $ = (id) => document.getElementById(id);
  const state = { inFlight: false, timer: null, refreshMs: 5000 };
  const text = (id, value) => { const el = $(id); if (el) el.textContent = value; };
  const number = (value, digits = 0) => Number.isFinite(value) ? Number(value).toFixed(digits) : '—';
  const dateText = (value) => { const d = new Date(value); return Number.isNaN(d.getTime()) ? '—' : new Intl.DateTimeFormat('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(d); };
  const duration = (seconds) => { if (!Number.isFinite(seconds) || seconds < 0) return '—'; const days=Math.floor(seconds/86400), hours=Math.floor(seconds%86400/3600), mins=Math.floor(seconds%3600/60); return days ? `${days} ngày ${hours} giờ` : hours ? `${hours} giờ ${mins} phút` : `${mins} phút`; };
  const fetchJson = async (path) => { const response = await fetch(`${API}${path}`, { headers: { Accept: 'application/json' }, cache: 'no-store' }); if (!response.ok) throw new Error(`HTTP ${response.status}`); return response.json(); };
  const get = async (path) => { try { return await fetchJson(path); } catch { return null; } };
  const setConnection = (status, label) => { $('connection').dataset.state = status; text('connection-label', label); };
  const tempText = (v) => Number.isFinite(v) ? `${number(v,1)} °C` : 'Chưa có dữ liệu';

  function renderThermal(thermal, history) {
    const sensors = Array.isArray(thermal?.sensors) ? thermal.sensors : [];
    const values = sensors.filter(s => s?.available && Number.isFinite(s.value_celsius)).map(s => s.value_celsius);
    const high = values.length ? Math.max(...values) : null;
    text('thermal-high', `Cao nhất: ${high === null ? '—' : tempText(high)}`);
    text('thermal-state', high === null ? (sensors.length ? 'Cảm biến chưa sẵn sàng' : 'Chưa có dữ liệu') : high >= 45 ? 'Nhiệt độ cao' : high >= 38 ? 'Hơi ấm' : 'Bình thường');
    text('thermal-pill', high === null ? '—' : high >= 45 ? 'Cần chú ý' : high >= 38 ? 'Hơi ấm' : 'Ổn định');
    const records = Array.isArray(history?.records) ? history.records.filter(r => Number.isFinite(r?.value_celsius)) : [];
    const path = $('thermal-chart').querySelector('.chart-line');
    if (records.length > 1) {
      const series = records.slice(-60).map(r => r.value_celsius), min=Math.min(...series), max=Math.max(...series), span=Math.max(max-min,1);
      const points = series.map((v,i) => `${(i/(series.length-1))*600},${130-((v-min)/span)*100}`).join(' ');
      path.setAttribute('d', `M${points.replaceAll(' ', ' L')}`); $('thermal-chart-wrap').classList.add('has-data'); $('thermal-chart').setAttribute('aria-label', `Biểu đồ nhiệt độ gồm ${series.length} điểm lịch sử`); text('thermal-chart-label', `Lịch sử · ${series.length} điểm`);
    } else { path.setAttribute('d',''); $('thermal-chart-wrap').classList.remove('has-data'); $('thermal-chart').setAttribute('aria-label','Lịch sử nhiệt độ chưa có dữ liệu'); text('thermal-chart-label','Chưa có lịch sử'); }
  }
  function renderBattery(battery) {
    const level = Number.isInteger(battery?.level) && battery.level >= 0 && battery.level <= 100 ? battery.level : null;
    text('battery-level', level === null ? '—' : String(level));
    text('battery-status', battery?.charging_status ? battery.charging_status.replaceAll('_',' ') : 'Chưa có dữ liệu');
    text('battery-temp', tempText(battery?.temperature_celsius));
    text('battery-voltage', Number.isFinite(battery?.voltage_mv) ? `${number(battery.voltage_mv)} mV` : '—');
    text('battery-health', battery?.health || '—');
    $('battery-progress').setAttribute('aria-valuenow', level ?? 0); $('battery-progress-fill').style.width = `${level ?? 0}%`;
  }
  function renderAlerts(data) {
    const alerts = Array.isArray(data?.alerts) ? data.alerts : [];
    const critical = alerts.filter(a => a?.severity === 'critical').length, warning = alerts.filter(a => a?.severity === 'warning').length;
    text('alert-summary', alerts.length ? `${critical} nghiêm trọng · ${warning} cảnh báo` : data ? 'Không có cảnh báo' : 'Chưa có dữ liệu');
    const host = $('alert-list'); host.replaceChildren();
    if (!alerts.length) { const p=document.createElement('p'); p.className='empty-state'; p.textContent=data?'Không có cảnh báo đang hoạt động.':'Chưa có cảnh báo từ backend.'; host.append(p); return; }
    for (const alert of alerts.slice(0,4)) {
      const row=document.createElement('div'); row.className='alert-item'; const severity=document.createElement('span'); severity.className=`severity ${['critical','warning','info'].includes(alert.severity)?alert.severity:'info'}`; severity.textContent=(alert.severity||'info').toUpperCase();
      const copy=document.createElement('div'); copy.className='alert-copy'; const title=document.createElement('strong'); title.textContent=alert.message||alert.metric||'Cảnh báo'; const meta=document.createElement('small'); meta.textContent=[alert.metric, Number.isFinite(alert.observed_value)?`Giá trị ${alert.observed_value}`:null, alert.timestamp?dateText(alert.timestamp):null].filter(Boolean).join(' · '); copy.append(title,meta); row.append(severity,copy); host.append(row);
    }
  }
  function renderSensors(data) {
    const list=Array.isArray(data?.sensors)?data.sensors:[]; text('sensor-count',list.length?`${list.filter(s=>s?.available).length} / ${list.length}`:'—'); text('sensor-badge',String(list.length||'—'));
    const host=$('sensor-list'); host.replaceChildren();
    if (!list.length) { const p=document.createElement('p'); p.className='empty-state'; p.textContent='Danh sách cảm biến sẽ xuất hiện khi backend kết nối.'; host.append(p); return; }
    for (const sensor of list) { const row=document.createElement('div'); row.className='sensor-row'; const name=document.createElement('span'); name.className='sensor-name'; name.textContent=sensor?.name||'Cảm biến không tên'; const kind=document.createElement('span'); kind.className='sensor-type'; kind.textContent=sensor?.category||'—'; const value=document.createElement('span'); value.className=`sensor-value ${sensor?.available?'available':'unavailable'}`; value.textContent=sensor?.available&&Number.isFinite(sensor.last_value)?`${sensor.last_value} ${sensor.unit||''}`.trim():sensor?.available?'Sẵn sàng':'Không khả dụng'; row.append(name,kind,value); host.append(row); }
  }
  async function refresh() {
    if (state.inFlight || document.hidden) return; state.inFlight=true; $('refresh-button').disabled=true;
    const [health,system,battery,thermal,sensors,alerts,thermalHistory] = await Promise.all(['/health','/system','/battery','/thermal','/sensors','/alerts','/history/thermal?limit=60'].map(get));
    const online=Boolean(health); setConnection(online?'online':'offline',online?'Backend trực tuyến':'Backend ngoại tuyến'); $('api-notice').hidden=online;
    if (system) { if (system.device_name) text('device-name',system.device_name); if (system.android_version) text('android-version',system.android_version); text('uptime',duration(system.uptime_seconds)); }
    if (battery) renderBattery(battery); if (thermal) renderThermal(thermal,thermalHistory); renderSensors(sensors); renderAlerts(alerts);
    const timestamp=battery?.timestamp||system?.timestamp||health?.timestamp; if (timestamp) { const formatted=dateText(timestamp); text('last-updated',formatted); text('footer-updated',`Cập nhật ${formatted}`); }
    text('agent-status',online?'Đang hoạt động':'Đang chờ backend'); state.inFlight=false; $('refresh-button').disabled=false;
  }
  function schedule() { clearTimeout(state.timer); if (!document.hidden) state.timer=setTimeout(async()=>{await refresh(); schedule();},state.refreshMs); }
  $('refresh-button').addEventListener('click',async()=>{await refresh();schedule();});
  document.addEventListener('visibilitychange',()=>{clearTimeout(state.timer);if(!document.hidden){refresh().finally(schedule);}});
  document.querySelectorAll('.nav-item').forEach(link=>link.addEventListener('click',()=>{document.querySelectorAll('.nav-item').forEach(item=>{item.classList.remove('active');item.removeAttribute('aria-current');});link.classList.add('active');link.setAttribute('aria-current','page');}));
  refresh().finally(schedule);
})();
