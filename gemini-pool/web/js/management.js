// 管理页面 JavaScript
class ApiKeyManagement {
    constructor() {
        this.messageContainer = document.getElementById('message-container');
        this.apiKeysTable = document.getElementById('apiKeysTable');
        this.createModal = document.getElementById('createModal');
        this.editModal = document.getElementById('editModal');
        this.createForm = document.getElementById('createApiKeyForm');
        this.editForm = document.getElementById('editApiKeyForm');
        this.toggleKeyBtn = document.getElementById('toggleKeyVisibility');
        
        this.token = localStorage.getItem('adminToken');
        this.currentApiKey = null; // 存储当前编辑的完整API key
        
        this.init();
    }
    
    init() {
        // 检查认证状态
        if (!this.token) {
            window.location.href = 'login.html';
            return;
        }
        
        // 初始化多语言支持
        window.i18n.init();
        
        // 绑定表单事件
        this.createForm.addEventListener('submit', (e) => this.handleCreateApiKey(e));
        this.editForm.addEventListener('submit', (e) => this.handleEditApiKey(e));
        
        // 绑定API key显示/隐藏按钮事件
        if (this.toggleKeyBtn) {
            this.toggleKeyBtn.addEventListener('click', () => this.toggleApiKeyVisibility());
        }
        
        // 加载数据
        this.loadDashboardData();
        this.loadApiKeys();
        
        // 定期刷新数据
        setInterval(() => {
            this.loadDashboardData();
            this.loadApiKeys();
        }, 30000); // 每30秒刷新一次
        
        // 监听语言切换事件
        window.addEventListener('languageChanged', () => {
            this.updateButtonTexts();
            this.loadApiKeys(); // 重新渲染表格以更新状态文本
        });
    }
    
    async loadDashboardData() {
        try {
            const response = await fetch('/admin/api/dashboard', {
                headers: {
                    'Authorization': `Bearer ${this.token}`
                }
            });
            
            if (response.ok) {
                const data = await response.json();
                this.updateDashboard(data);
            } else if (response.status === 401) {
                this.handleAuthError();
            }
        } catch (error) {
            console.error('Error loading dashboard data:', error);
        }
    }
    
    async loadApiKeys() {
        try {
            const response = await fetch('/admin/api/api-keys', {
                headers: {
                    'Authorization': `Bearer ${this.token}`
                }
            });
            
            if (response.ok) {
                const data = await response.json();
                this.renderApiKeysTable(data.api_keys);
            } else if (response.status === 401) {
                this.handleAuthError();
            }
        } catch (error) {
            console.error('Error loading API keys:', error);
            this.showMessage('加载 API Keys 失败', 'error');
        }
    }
    
    updateDashboard(data) {
        document.getElementById('totalApiKeys').textContent = data.total_api_keys || 0;
        document.getElementById('totalRequests').textContent = window.i18n.formatNumber(data.total_requests || 0);
        document.getElementById('totalTokens').textContent = window.i18n.formatNumber(data.total_tokens || 0);
        document.getElementById('activeKeys').textContent = data.active_keys || 0;
    }
    
    renderApiKeysTable(apiKeys) {
        this.apiKeysTable.innerHTML = '';
        
        apiKeys.forEach(key => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${this.escapeHtml(key.key_name)}</td>
                <td>
                    <span class="api-key-display">
                        ${this.maskApiKey(key.api_key)}
                    </span>
                </td>
                <td>
                    <span class="status-badge ${key.is_active ? 'status-active' : 'status-inactive'}">
                        ${key.is_active ? window.i18n.t('active') : window.i18n.t('inactive')}
                    </span>
                </td>
                <td>${window.i18n.formatDate(key.created_at)}</td>
                <td>${window.i18n.formatNumber(key.total_requests)}</td>
                <td>${window.i18n.formatNumber(key.total_input_tokens)}</td>
                <td>${window.i18n.formatNumber(key.total_output_tokens)}</td>
                <td>
                    <button class="btn" style="margin-right: 5px; padding: 8px 15px; font-size: 0.8rem;" 
                            onclick="management.showEditModal('${key.id}')">
                        ${window.i18n.t('edit')}
                    </button>
                    <button class="btn btn-danger" style="padding: 8px 15px; font-size: 0.8rem;" 
                            onclick="management.deleteApiKey('${key.id}')" 
                            ${!key.is_active ? 'style="opacity: 0.5;"' : ''}>
                        ${window.i18n.t('delete')}
                    </button>
                </td>
            `;
            this.apiKeysTable.appendChild(row);
        });
    }
    
    async handleCreateApiKey(e) {
        e.preventDefault();
        
        const formData = new FormData(this.createForm);
        const keyName = formData.get('keyName').trim();
        const keyValue = formData.get('keyValue').trim();
        
        if (!keyName) {
            this.showMessage(window.i18n.t('missing_api_key_name'), 'error');
            return;
        }
        
        this.setCreateLoading(true);
        
        try {
            const response = await fetch('/admin/api/api-keys', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    key_name: keyName,
                    api_key: keyValue || undefined
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                this.showMessage(window.i18n.t('api_key_created'), 'success');
                this.hideCreateModal();
                this.loadApiKeys();
                this.loadDashboardData();
                
                // 如果是自动生成的 key，显示完整的 key
                if (data.api_key) {
                    this.showMessage(`${window.i18n.t('new_api_key')}: ${data.api_key}`, 'warning');
                }
            } else {
                this.showMessage(data.error || window.i18n.t('api_key_creation_failed'), 'error');
            }
        } catch (error) {
            console.error('Error creating API key:', error);
            this.showMessage(window.i18n.t('network_error'), 'error');
        }
        
        this.setCreateLoading(false);
    }
    
    async handleEditApiKey(e) {
        e.preventDefault();
        
        const formData = new FormData(this.editForm);
        const keyId = formData.get('keyId');
        const keyName = formData.get('keyName').trim();
        const isActive = formData.get('isActive') === 'true';
        
        this.setEditLoading(true);
        
        try {
            const response = await fetch(`/admin/api/api-keys/${keyId}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    key_name: keyName,
                    is_active: isActive
                })
            });
            
            if (response.ok) {
                this.showMessage(window.i18n.t('api_key_updated'), 'success');
                this.hideEditModal();
                this.loadApiKeys();
                this.loadDashboardData();
            } else {
                const data = await response.json();
                this.showMessage(data.error || window.i18n.t('api_key_update_failed'), 'error');
            }
        } catch (error) {
            console.error('Error updating API key:', error);
            this.showMessage(window.i18n.t('network_error'), 'error');
        }
        
        this.setEditLoading(false);
    }
    
    async deleteApiKey(keyId) {
        if (!confirm(window.i18n.t('delete_api_key_confirm'))) {
            return;
        }
        
        try {
            const response = await fetch(`/admin/api/api-keys/${keyId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.token}`
                }
            });
            
            if (response.ok) {
                this.showMessage(window.i18n.t('api_key_deleted'), 'success');
                this.loadApiKeys();
                this.loadDashboardData();
            } else {
                const data = await response.json();
                this.showMessage(data.error || window.i18n.t('api_key_delete_failed'), 'error');
            }
        } catch (error) {
            console.error('Error deleting API key:', error);
            this.showMessage(window.i18n.t('network_error'), 'error');
        }
    }
    
    async showEditModal(keyId) {
        try {
            const response = await fetch(`/admin/api/api-keys/${keyId}`, {
                headers: {
                    'Authorization': `Bearer ${this.token}`
                }
            });
            
            if (response.ok) {
                const keyData = await response.json();
                
                document.getElementById('editKeyId').value = keyData.id;
                document.getElementById('editKeyName').value = keyData.key_name;
                
                // 存储完整的API key并初始显示为掩码
                this.currentApiKey = keyData.api_key;
                this.updateApiKeyDisplay(false); // 初始状态为隐藏
                
                if (keyData.is_active) {
                    document.getElementById('editActiveTrue').checked = true;
                } else {
                    document.getElementById('editActiveFalse').checked = true;
                }
                
                this.editModal.style.display = 'block';
                this.editModal.classList.add('show');
            } else {
                this.showMessage(window.i18n.t('load_failed'), 'error');
            }
        } catch (error) {
            console.error('Error loading API key details:', error);
            this.showMessage(window.i18n.t('network_error'), 'error');
        }
    }
    
    showCreateModal() {
        this.createForm.reset();
        this.createModal.style.display = 'block';
        this.createModal.classList.add('show');
    }
    
    hideCreateModal() {
        this.createModal.style.display = 'none';
        this.createModal.classList.remove('show');
        this.createForm.reset();
    }
    
    hideEditModal() {
        this.editModal.style.display = 'none';
        this.editModal.classList.remove('show');
        this.editForm.reset();
        // 清除单选按钮的选中状态
        document.getElementById('editActiveTrue').checked = false;
        document.getElementById('editActiveFalse').checked = false;
        // 清除API key相关状态
        this.currentApiKey = null;
        document.getElementById('editKeyValue').value = '';
        this.updateToggleButtonText(false);
    }
    
    setCreateLoading(loading) {
        const btn = document.getElementById('createBtnText');
        const spinner = document.getElementById('createBtnLoading');
        
        if (loading) {
            btn.style.display = 'none';
            spinner.style.display = 'inline-block';
        } else {
            btn.style.display = 'inline';
            spinner.style.display = 'none';
        }
    }
    
    setEditLoading(loading) {
        const btn = document.getElementById('editBtnText');
        const spinner = document.getElementById('editBtnLoading');
        
        if (loading) {
            btn.style.display = 'none';
            spinner.style.display = 'inline-block';
        } else {
            btn.style.display = 'inline';
            spinner.style.display = 'none';
        }
    }
    
    showMessage(message, type) {
        this.messageContainer.innerHTML = `
            <div class="message message-${type}">
                ${message}
            </div>
        `;
        
        setTimeout(() => {
            this.messageContainer.innerHTML = '';
        }, 5000);
    }
    
    handleAuthError() {
        localStorage.removeItem('adminToken');
        window.location.href = 'login.html';
    }
    
    formatNumber(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1) + 'M';
        } else if (num >= 1000) {
            return (num / 1000).toFixed(1) + 'K';
        }
        return num.toString();
    }
    
    updateButtonTexts() {
        // 更新创建按钮文本
        const createBtnText = document.getElementById('createBtnText');
        if (createBtnText) {
            createBtnText.textContent = window.i18n.t('create_api_key');
        }
        
        // 更新编辑按钮文本
        const editBtnText = document.getElementById('editBtnText');
        if (editBtnText) {
            editBtnText.textContent = window.i18n.t('save_changes');
        }
    }
    
    
    maskApiKey(apiKey) {
        if (apiKey.length <= 8) return apiKey;
        return apiKey.substring(0, 4) + '****' + apiKey.substring(apiKey.length - 4);
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    toggleApiKeyVisibility() {
        const keyInput = document.getElementById('editKeyValue');
        const isCurrentlyMasked = keyInput.value.includes('****');
        this.updateApiKeyDisplay(!isCurrentlyMasked);
    }
    
    updateApiKeyDisplay(showFull) {
        const keyInput = document.getElementById('editKeyValue');
        if (showFull && this.currentApiKey) {
            keyInput.value = this.currentApiKey;
        } else if (this.currentApiKey) {
            keyInput.value = this.maskApiKey(this.currentApiKey);
        }
        this.updateToggleButtonText(showFull);
    }
    
    updateToggleButtonText(isShowing) {
        const toggleBtn = document.getElementById('toggleKeyVisibility');
        const btnText = toggleBtn.querySelector('span');
        if (isShowing) {
            btnText.textContent = window.i18n.t('hide') || '隐藏';
            btnText.setAttribute('data-i18n', 'hide');
        } else {
            btnText.textContent = window.i18n.t('show') || '显示';
            btnText.setAttribute('data-i18n', 'show');
        }
    }
}

// 全局函数，供 HTML 调用
function showCreateModal() {
    management.showCreateModal();
}

function hideCreateModal() {
    management.hideCreateModal();
}

function hideEditModal() {
    management.hideEditModal();
}

function logout() {
    localStorage.removeItem('adminToken');
    window.location.href = 'login.html';
}

// 页面加载完成后初始化管理功能
let management;
document.addEventListener('DOMContentLoaded', () => {
    management = new ApiKeyManagement();
});