use axum::response::Html;

pub async fn admin_page() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

static ADMIN_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>河北区县管理</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f6fa; color: #333; }
.header { background: #fff; padding: 16px 24px; box-shadow: 0 1px 4px rgba(0,0,0,0.08); display: flex; justify-content: space-between; align-items: center; position: sticky; top: 0; z-index: 100; }
.header h1 { font-size: 18px; font-weight: 600; }
.btn { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; transition: all .2s; }
.btn-primary { background: #4f46e5; color: #fff; }
.btn-primary:hover { background: #4338ca; }
.btn-success { background: #059669; color: #fff; }
.btn-success:hover { background: #047857; }
.btn-danger { background: #dc2626; color: #fff; }
.btn-danger:hover { background: #b91c1c; }
.btn-sm { padding: 4px 10px; font-size: 12px; }
.container { max-width: 1200px; margin: 20px auto; padding: 0 16px; }
.toolbar { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
.filter-select { padding: 8px 12px; border: 1px solid #ddd; border-radius: 6px; font-size: 14px; min-width: 140px; }
.stats { background: #fff; border-radius: 8px; padding: 16px 20px; margin-bottom: 16px; display: flex; gap: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
.stat-item { display: flex; flex-direction: column; }
.stat-num { font-size: 24px; font-weight: 700; color: #4f46e5; }
.stat-label { font-size: 12px; color: #999; margin-top: 2px; }
.card { background: #fff; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); margin-bottom: 12px; overflow: hidden; transition: box-shadow .2s; }
.card:hover { box-shadow: 0 4px 12px rgba(0,0,0,0.1); }
.card-body { padding: 16px 20px; display: flex; gap: 16px; align-items: flex-start; }
.card-img { width: 120px; height: 80px; border-radius: 6px; object-fit: cover; flex-shrink: 0; background: #eee; }
.card-no-img { width: 120px; height: 80px; border-radius: 6px; background: #f0f0f0; display: flex; align-items: center; justify-content: center; color: #bbb; font-size: 12px; flex-shrink: 0; }
.card-info { flex: 1; min-width: 0; }
.card-title { font-size: 15px; font-weight: 600; margin-bottom: 4px; }
.card-city { font-size: 12px; color: #999; margin-bottom: 6px; }
.card-desc { font-size: 13px; color: #666; line-height: 1.6; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.card-actions { display: flex; gap: 6px; flex-shrink: 0; align-items: flex-start; }
.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); z-index: 1000; display: flex; align-items: center; justify-content: center; }
.modal { background: #fff; border-radius: 12px; padding: 24px; width: 90%; max-width: 520px; max-height: 90vh; overflow-y: auto; }
.modal h2 { font-size: 18px; margin-bottom: 20px; }
.form-group { margin-bottom: 16px; }
.form-group label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 6px; color: #555; }
.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 10px 12px; border: 1px solid #ddd; border-radius: 6px; font-size: 14px; font-family: inherit; }
.form-group textarea { min-height: 100px; resize: vertical; }
.form-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.upload-area { border: 2px dashed #ddd; border-radius: 8px; padding: 20px; text-align: center; cursor: pointer; transition: border-color .2s; }
.upload-area:hover { border-color: #4f46e5; }
.upload-area img { max-width: 100%; max-height: 200px; border-radius: 6px; margin-bottom: 8px; }
.empty { text-align: center; padding: 60px 20px; color: #999; }
</style>
</head>
<body>
<div class="header">
    <h1>河北区县简介管理</h1>
    <div style="display:flex;gap:10px;">
        <button class="btn btn-success" id="syncBtn" onclick="initData()">同步河北行政规划</button>
        <button class="btn btn-primary" onclick="openCreateModal()">+ 新增区县</button>
    </div>
</div>
<div class="container">
    <div class="stats">
        <div class="stat-item"><span class="stat-num" id="totalCount">0</span><span class="stat-label">总区县数</span></div>
        <div class="stat-item"><span class="stat-num" id="hasImgCount">0</span><span class="stat-label">已配图</span></div>
        <div class="stat-item"><span class="stat-num" id="noImgCount">0</span><span class="stat-label">未配图</span></div>
    </div>
    <div class="toolbar">
        <select class="filter-select" id="cityFilter" onchange="loadList()">
            <option value="">全部城市</option>
        </select>
        <input type="text" class="filter-select" id="searchInput" placeholder="搜索区县名称..." oninput="loadList()">
    </div>
    <div id="listContainer"></div>
</div>
<div id="modalContainer"></div>
<script>
const API = '/district/api';
const API_KEY = 'jjjshop-district-2026';
let allData = [];

async function api(path, opts = {}) {
    const headers = { 'Content-Type': 'application/json', 'X-API-Key': API_KEY, ...(opts.headers || {}) };
    const res = await fetch(API + path, { ...opts, headers });
    return res.json();
}

async function loadData() {
    const res = await api('/districts');
    if (res.code === 0) {
        allData = res.data || [];
        updateCities();
        updateStats();
        loadList();
    }
}

function updateCities() {
    const cities = [...new Set(allData.map(d => d.city))];
    const sel = document.getElementById('cityFilter');
    const val = sel.value;
    sel.innerHTML = '<option value="">全部城市</option>' + cities.map(c => `<option value="${c}">${c}</option>`).join('');
    sel.value = val;
}

function updateStats() {
    document.getElementById('totalCount').textContent = allData.length;
    document.getElementById('hasImgCount').textContent = allData.filter(d => d.image).length;
    document.getElementById('noImgCount').textContent = allData.filter(d => !d.image).length;
}

function loadList() {
    const city = document.getElementById('cityFilter').value;
    const search = document.getElementById('searchInput').value.trim();
    let filtered = allData;
    if (city) filtered = filtered.filter(d => d.city === city);
    if (search) filtered = filtered.filter(d => d.area.includes(search) || d.description.includes(search));
    const container = document.getElementById('listContainer');
    if (filtered.length === 0) {
        container.innerHTML = '<div class="empty">暂无数据</div>';
        return;
    }
    container.innerHTML = filtered.map(d => `
        <div class="card">
            <div class="card-body">
                ${d.image ? `<img class="card-img" src="${d.image}" alt="${d.area}">` : '<div class="card-no-img">暂无图片</div>'}
                <div class="card-info">
                    <div class="card-title">${d.area}</div>
                    <div class="card-city">${d.city}</div>
                    <div class="card-desc">${d.description || '暂无简介'}</div>
                </div>
                <div class="card-actions">
                    <button class="btn btn-primary btn-sm" onclick='openEditModal(${JSON.stringify(d).replace(/'/g,"&#39;")})'>编辑</button>
                    <button class="btn btn-danger btn-sm" onclick="deleteDistrict('${d.city}','${d.area}')">删除</button>
                </div>
            </div>
        </div>
    `).join('');
}

function closeModal() { document.getElementById('modalContainer').innerHTML = ''; }

function openCreateModal() {
    const cities = ['石家庄市','唐山市','秦皇岛市','邯郸市','保定市','张家口市','承德市','廊坊市','沧州市','衡水市','邢台市'];
    document.getElementById('modalContainer').innerHTML = `
        <div class="modal-overlay" onclick="if(event.target===this)closeModal()">
            <div class="modal">
                <h2>新增区县</h2>
                <div class="form-group">
                    <label>所属城市</label>
                    <select id="f_city">${cities.map(c=>`<option value="${c}">${c}</option>`).join('')}</select>
                </div>
                <div class="form-group"><label>区县名称</label><input id="f_area" placeholder="如：长安区"></div>
                <div class="form-group"><label>简介</label><textarea id="f_desc" placeholder="请输入区县简介"></textarea></div>
                <div class="form-group">
                    <label>图片</label>
                    <div class="upload-area" onclick="document.getElementById('f_file').click()">
                        <div id="f_preview">点击上传图片</div>
                        <input type="file" id="f_file" accept="image/*" style="display:none" onchange="previewFile(this,'f_preview')">
                    </div>
                </div>
                <div class="form-actions">
                    <button class="btn" onclick="closeModal()" style="background:#eee">取消</button>
                    <button class="btn btn-primary" onclick="createDistrict()">创建</button>
                </div>
            </div>
        </div>`;
}

function openEditModal(d) {
    document.getElementById('modalContainer').innerHTML = `
        <div class="modal-overlay" onclick="if(event.target===this)closeModal()">
            <div class="modal">
                <h2>编辑 ${d.area}</h2>
                <div class="form-group"><label>所属城市</label><input id="f_city" value="${d.city}" disabled></div>
                <div class="form-group"><label>区县名称</label><input id="f_area" value="${d.area}" disabled></div>
                <div class="form-group"><label>简介</label><textarea id="f_desc">${d.description}</textarea></div>
                <div class="form-group">
                    <label>图片</label>
                    <div class="upload-area" onclick="document.getElementById('f_file').click()">
                        <div id="f_preview">${d.image ? `<img src="${d.image}">` : '点击上传图片'}</div>
                        <input type="file" id="f_file" accept="image/*" style="display:none" onchange="previewFile(this,'f_preview')">
                    </div>
                </div>
                <div class="form-actions">
                    <button class="btn" onclick="closeModal()" style="background:#eee">取消</button>
                    <button class="btn btn-primary" onclick="updateDistrict('${d.city}','${d.area}')">保存</button>
                </div>
            </div>
        </div>`;
}

function previewFile(input, previewId) {
    const file = input.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = e => {
        document.getElementById(previewId).innerHTML = `<img src="${e.target.result}">`;
    };
    reader.readAsDataURL(file);
}

async function createDistrict() {
    const city = document.getElementById('f_city').value;
    const area = document.getElementById('f_area').value.trim();
    const desc = document.getElementById('f_desc').value.trim();
    if (!area) { alert('请输入区县名称'); return; }
    const res = await api('/districts', { method: 'POST', body: JSON.stringify({ city, area, description: desc, image: '' }) });
    if (res.code !== 0) { alert(res.msg); return; }
    const fileInput = document.getElementById('f_file');
    if (fileInput.files[0]) {
        const fd = new FormData();
        fd.append('file', fileInput.files[0]);
        await fetch(`${API}/districts/${encodeURIComponent(city)}/${encodeURIComponent(area)}/image`, { method: 'POST', body: fd, headers: { 'X-API-Key': API_KEY } });
    }
    closeModal();
    loadData();
}

async function updateDistrict(city, area) {
    const desc = document.getElementById('f_desc').value.trim();
    const res = await api(`/districts/${encodeURIComponent(city)}/${encodeURIComponent(area)}`, {
        method: 'PUT', body: JSON.stringify({ description: desc, image: '' })
    });
    if (res.code !== 0) { alert(res.msg); return; }
    const fileInput = document.getElementById('f_file');
    if (fileInput.files[0]) {
        const fd = new FormData();
        fd.append('file', fileInput.files[0]);
        await fetch(`${API}/districts/${encodeURIComponent(city)}/${encodeURIComponent(area)}/image`, { method: 'POST', body: fd, headers: { 'X-API-Key': API_KEY } });
    }
    closeModal();
    loadData();
}

async function deleteDistrict(city, area) {
    if (!confirm(`确定删除 ${area} 吗？`)) return;
    await api(`/districts/${encodeURIComponent(city)}/${encodeURIComponent(area)}`, { method: 'DELETE' });
    loadData();
}

async function initData() {
    if (!confirm('将同步河北省的行政规划，已存在的不会被覆盖。继续吗？')) return;
    const btn = document.getElementById('syncBtn');
    btn.disabled = true;
    btn.textContent = '同步中...';
    try {
        const res = await api('/init-hebei', { method: 'POST' });
        alert(res.data || res.msg);
        loadData();
    } catch (e) {
        alert('同步失败: ' + e.message);
    } finally {
        btn.disabled = false;
        btn.textContent = '同步河北行政规划';
    }
}

loadData();
</script>
</body>
</html>"##;
