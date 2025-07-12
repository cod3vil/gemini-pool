// 国际化多语言支持模块
class I18n {
    constructor() {
        this.currentLang = localStorage.getItem('language') || 'zh';
        this.translations = {
            zh: {
                // 通用
                'login': '登录',
                'logout': '退出登录',
                'cancel': '取消',
                'save': '保存',
                'delete': '删除',
                'edit': '编辑',
                'create': '创建',
                'confirm': '确认',
                'loading': '加载中...',
                'success': '成功',
                'error': '错误',
                'warning': '警告',
                'total': '总计',
                'active': '活跃',
                'inactive': '禁用',
                'status': '状态',
                'name': '名称',
                'actions': '操作',
                'created_at': '创建时间',
                
                // 登录页面
                'login_title': 'GEMINI POOL',
                'login_subtitle': '管理员控制台',
                'username': '用户名',
                'password': '密码',
                'username_placeholder': '输入管理员用户名',
                'password_placeholder': '输入管理员密码',
                'login_button': '登录系统',
                'login_success': '登录成功，正在跳转...',
                'login_failed': '登录失败',
                'invalid_credentials': '用户名或密码错误',
                'missing_credentials': '请输入用户名和密码',
                'network_error': '网络错误，请稍后重试',
                'ai_powered': 'AI-Powered API Gateway',
                'version': 'Version 1.0.0',
                
                // 管理页面
                'management_title': 'GEMINI POOL 控制台',
                'admin_user': '管理员',
                'dashboard': '仪表板',
                'api_keys': 'API Keys',
                'requests': '请求数',
                'tokens': 'Token 数',
                'active_keys': '活跃 Keys',
                'input_tokens': '输入 Tokens',
                'output_tokens': '输出 Tokens',
                'total_requests': '总请求数',
                'total_tokens': '总 Token 数',
                
                // API Key 管理
                'api_key_management': 'API Keys 管理',
                'create_new_api_key': '+ 创建新 API Key',
                'api_key': 'API Key',
                'api_key_name': 'Key 名称',
                'api_key_value': 'API Key 值',
                'api_key_placeholder': '输入 API Key 名称',
                'api_key_value_placeholder': '输入 API Key 值，留空自动生成',
                'api_key_auto_generate': '留空将自动生成一个安全的 API Key',
                'create_api_key': '创建 API Key',
                'edit_api_key': '编辑 API Key',
                'delete_api_key_confirm': '确定要删除这个 API Key 吗？此操作不可恢复。',
                'api_key_created': 'API Key 创建成功',
                'api_key_updated': 'API Key 更新成功',
                'api_key_deleted': 'API Key 删除成功',
                'api_key_creation_failed': 'API Key 创建失败',
                'api_key_update_failed': 'API Key 更新失败',
                'api_key_delete_failed': 'API Key 删除失败',
                'api_key_not_found': 'API Key 不存在',
                'new_api_key': '新 API Key',
                'save_changes': '保存更改',
                'enable': '启用',
                'disable': '禁用',
                'show': '显示',
                'hide': '隐藏',
                
                // 表格标题
                'table_name': '名称',
                'table_api_key': 'API Key',
                'table_status': '状态',
                'table_created_at': '创建时间',
                'table_requests': '请求数',
                'table_input_tokens': '输入 Tokens',
                'table_output_tokens': '输出 Tokens',
                'table_actions': '操作',
                
                // 错误信息
                'missing_api_key_name': '请输入 API Key 名称',
                'invalid_api_key': '无效的 API Key',
                'unauthorized': '未授权访问',
                'session_expired': '会话已过期，请重新登录',
                'load_failed': '加载失败',
                'operation_failed': '操作失败',
                
                // 语言切换
                'language': '语言',
                'chinese': '中文',
                'english': 'English'
            },
            en: {
                // Common
                'login': 'Login',
                'logout': 'Logout',
                'cancel': 'Cancel',
                'save': 'Save',
                'delete': 'Delete',
                'edit': 'Edit',
                'create': 'Create',
                'confirm': 'Confirm',
                'loading': 'Loading...',
                'success': 'Success',
                'error': 'Error',
                'warning': 'Warning',
                'total': 'Total',
                'active': 'Active',
                'inactive': 'Inactive',
                'status': 'Status',
                'name': 'Name',
                'actions': 'Actions',
                'created_at': 'Created At',
                
                // Login page
                'login_title': 'GEMINI POOL',
                'login_subtitle': 'Admin Console',
                'username': 'Username',
                'password': 'Password',
                'username_placeholder': 'Enter admin username',
                'password_placeholder': 'Enter admin password',
                'login_button': 'Login System',
                'login_success': 'Login successful, redirecting...',
                'login_failed': 'Login failed',
                'invalid_credentials': 'Invalid username or password',
                'missing_credentials': 'Please enter username and password',
                'network_error': 'Network error, please try again later',
                'ai_powered': 'AI-Powered API Gateway',
                'version': 'Version 1.0.0',
                
                // Management page
                'management_title': 'GEMINI POOL Console',
                'admin_user': 'Admin',
                'dashboard': 'Dashboard',
                'api_keys': 'API Keys',
                'requests': 'Requests',
                'tokens': 'Tokens',
                'active_keys': 'Active Keys',
                'input_tokens': 'Input Tokens',
                'output_tokens': 'Output Tokens',
                'total_requests': 'Total Requests',
                'total_tokens': 'Total Tokens',
                
                // API Key management
                'api_key_management': 'API Keys Management',
                'create_new_api_key': '+ Create New API Key',
                'api_key': 'API Key',
                'api_key_name': 'Key Name',
                'api_key_value': 'API Key Value',
                'api_key_placeholder': 'Enter API Key name',
                'api_key_value_placeholder': 'Enter API Key value, leave empty to auto-generate',
                'api_key_auto_generate': 'Leave empty to auto-generate a secure API Key',
                'create_api_key': 'Create API Key',
                'edit_api_key': 'Edit API Key',
                'delete_api_key_confirm': 'Are you sure you want to delete this API Key? This action cannot be undone.',
                'api_key_created': 'API Key created successfully',
                'api_key_updated': 'API Key updated successfully',
                'api_key_deleted': 'API Key deleted successfully',
                'api_key_creation_failed': 'Failed to create API Key',
                'api_key_update_failed': 'Failed to update API Key',
                'api_key_delete_failed': 'Failed to delete API Key',
                'api_key_not_found': 'API Key not found',
                'new_api_key': 'New API Key',
                'save_changes': 'Save Changes',
                'enable': 'Enable',
                'disable': 'Disable',
                'show': 'Show',
                'hide': 'Hide',
                
                // Table headers
                'table_name': 'Name',
                'table_api_key': 'API Key',
                'table_status': 'Status',
                'table_created_at': 'Created At',
                'table_requests': 'Requests',
                'table_input_tokens': 'Input Tokens',
                'table_output_tokens': 'Output Tokens',
                'table_actions': 'Actions',
                
                // Error messages
                'missing_api_key_name': 'Please enter API Key name',
                'invalid_api_key': 'Invalid API Key',
                'unauthorized': 'Unauthorized access',
                'session_expired': 'Session expired, please login again',
                'load_failed': 'Failed to load',
                'operation_failed': 'Operation failed',
                
                // Language switch
                'language': 'Language',
                'chinese': '中文',
                'english': 'English'
            }
        };
    }
    
    // 获取翻译文本
    t(key, defaultText = '') {
        const translation = this.translations[this.currentLang];
        return translation && translation[key] ? translation[key] : (defaultText || key);
    }
    
    // 切换语言
    switchLanguage(lang) {
        if (this.translations[lang]) {
            this.currentLang = lang;
            localStorage.setItem('language', lang);
            this.updatePageContent();
            this.dispatchLanguageChangeEvent();
        }
    }
    
    // 获取当前语言
    getCurrentLanguage() {
        return this.currentLang;
    }
    
    // 更新页面内容
    updatePageContent() {
        // 更新所有带有 data-i18n 属性的元素
        document.querySelectorAll('[data-i18n]').forEach(element => {
            const key = element.getAttribute('data-i18n');
            const translation = this.t(key);
            
            if (element.tagName === 'INPUT' && element.type === 'text' || element.type === 'password') {
                element.placeholder = translation;
            } else {
                element.textContent = translation;
            }
        });
        
        // 更新页面标题
        const titleElement = document.querySelector('title');
        if (titleElement && titleElement.getAttribute('data-i18n')) {
            titleElement.textContent = this.t(titleElement.getAttribute('data-i18n'));
        }
        
        // 更新语言选择器状态
        this.updateLanguageSwitcher();
    }
    
    // 更新语言切换器状态
    updateLanguageSwitcher() {
        const languageBtns = document.querySelectorAll('.lang-btn');
        languageBtns.forEach(btn => {
            btn.classList.remove('active');
            if (btn.getAttribute('data-lang') === this.currentLang) {
                btn.classList.add('active');
            }
        });
    }
    
    // 派发语言切换事件
    dispatchLanguageChangeEvent() {
        const event = new CustomEvent('languageChanged', {
            detail: { language: this.currentLang }
        });
        window.dispatchEvent(event);
    }
    
    // 初始化
    init() {
        this.createLanguageSwitcher();
        this.updatePageContent();
        
        // 监听语言切换事件
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('lang-btn')) {
                const lang = e.target.getAttribute('data-lang');
                this.switchLanguage(lang);
            }
        });
    }
    
    // 创建语言切换器
    createLanguageSwitcher() {
        const switcher = document.createElement('div');
        switcher.className = 'language-switcher';
        switcher.innerHTML = `
            <div class="language-selector">
                <div class="lang-options">
                    <button class="lang-btn" data-lang="zh" title="中文">中</button>
                    <button class="lang-btn" data-lang="en" title="English">EN</button>
                </div>
            </div>
        `;
        
        // 插入到页面右上角
        document.body.appendChild(switcher);
        this.updateLanguageSwitcher();
    }
    
    // 格式化数字（支持本地化）
    formatNumber(num) {
        if (this.currentLang === 'zh') {
            if (num >= 10000) {
                return (num / 10000).toFixed(1) + '万';
            } else if (num >= 1000) {
                return (num / 1000).toFixed(1) + '千';
            }
        } else {
            if (num >= 1000000) {
                return (num / 1000000).toFixed(1) + 'M';
            } else if (num >= 1000) {
                return (num / 1000).toFixed(1) + 'K';
            }
        }
        return num.toString();
    }
    
    // 格式化日期（支持本地化）
    formatDate(dateString) {
        const date = new Date(dateString);
        if (this.currentLang === 'zh') {
            return date.toLocaleDateString('zh-CN') + ' ' + date.toLocaleTimeString('zh-CN');
        } else {
            return date.toLocaleDateString('en-US') + ' ' + date.toLocaleTimeString('en-US');
        }
    }
}

// 创建全局 i18n 实例
window.i18n = new I18n();