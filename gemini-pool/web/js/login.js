// 登录页面 JavaScript
class Login {
    constructor() {
        this.form = document.getElementById('loginForm');
        this.usernameInput = document.getElementById('username');
        this.passwordInput = document.getElementById('password');
        this.loginBtn = document.getElementById('loginBtn');
        this.loginBtnText = document.getElementById('loginBtnText');
        this.loginBtnLoading = document.getElementById('loginBtnLoading');
        this.messageContainer = document.getElementById('message-container');
        
        this.init();
    }
    
    init() {
        // 初始化多语言支持
        window.i18n.init();
        
        // 绑定表单提交事件
        this.form.addEventListener('submit', (e) => this.handleLogin(e));
        
        // 绑定键盘事件
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                this.handleLogin(e);
            }
        });
        
        // 检查是否已经登录
        this.checkAuthStatus();
        
        // 添加输入框聚焦效果
        this.addInputEffects();
        
        // 监听语言切换事件
        window.addEventListener('languageChanged', () => {
            this.updateButtonText();
        });
    }
    
    addInputEffects() {
        [this.usernameInput, this.passwordInput].forEach(input => {
            input.addEventListener('focus', () => {
                input.parentElement.style.transform = 'scale(1.02)';
            });
            
            input.addEventListener('blur', () => {
                input.parentElement.style.transform = 'scale(1)';
            });
        });
    }
    
    async handleLogin(e) {
        e.preventDefault();
        
        const username = this.usernameInput.value.trim();
        const password = this.passwordInput.value.trim();
        
        if (!username || !password) {
            this.showMessage(window.i18n.t('missing_credentials'), 'error');
            return;
        }
        
        this.setLoading(true);
        
        try {
            const response = await fetch('/admin/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    username,
                    password
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                // 登录成功，保存 token
                localStorage.setItem('adminToken', data.token);
                this.showMessage(window.i18n.t('login_success'), 'success');
                
                // 延迟跳转，显示成功消息
                setTimeout(() => {
                    window.location.href = 'management.html';
                }, 1000);
            } else {
                this.showMessage(data.error || window.i18n.t('login_failed'), 'error');
            }
        } catch (error) {
            console.error('Login error:', error);
            this.showMessage(window.i18n.t('network_error'), 'error');
        }
        
        this.setLoading(false);
    }
    
    setLoading(loading) {
        if (loading) {
            this.loginBtn.disabled = true;
            this.loginBtnText.style.display = 'none';
            this.loginBtnLoading.style.display = 'inline-block';
        } else {
            this.loginBtn.disabled = false;
            this.loginBtnText.style.display = 'inline';
            this.loginBtnLoading.style.display = 'none';
        }
    }
    
    updateButtonText() {
        if (!this.loginBtn.disabled) {
            this.loginBtnText.textContent = window.i18n.t('login_button');
        }
    }
    
    showMessage(message, type) {
        this.messageContainer.innerHTML = `
            <div class="message message-${type}">
                ${message}
            </div>
        `;
        
        // 3秒后自动清除消息
        setTimeout(() => {
            this.messageContainer.innerHTML = '';
        }, 3000);
    }
    
    checkAuthStatus() {
        const token = localStorage.getItem('adminToken');
        if (token) {
            // 验证 token 是否有效
            fetch('/admin/api/auth/verify', {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            })
            .then(response => {
                if (response.ok) {
                    // Token 有效，直接跳转到管理页面
                    window.location.href = 'management.html';
                } else {
                    // Token 无效，清除本地存储
                    localStorage.removeItem('adminToken');
                }
            })
            .catch(() => {
                // 网络错误，清除本地存储
                localStorage.removeItem('adminToken');
            });
        }
    }
}

// 页面加载完成后初始化登录功能
document.addEventListener('DOMContentLoaded', () => {
    new Login();
});