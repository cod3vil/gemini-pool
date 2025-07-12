// 矩阵雨效果
class MatrixRain {
    constructor() {
        this.canvas = document.getElementById('matrix');
        this.ctx = this.canvas.getContext('2d');
        this.drops = [];
        this.chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@#$%^&*()*&^%+-/~{[|`]}';
        
        this.init();
        this.animate();
    }
    
    init() {
        this.resizeCanvas();
        window.addEventListener('resize', () => this.resizeCanvas());
        
        // 初始化雨滴
        const columns = Math.floor(this.canvas.width / 10);
        for (let i = 0; i < columns; i++) {
            this.drops[i] = 1;
        }
    }
    
    resizeCanvas() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }
    
    animate() {
        // 半透明背景，创建拖尾效果
        this.ctx.fillStyle = 'rgba(0, 0, 0, 0.04)';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // 设置文字样式
        this.ctx.fillStyle = '#0F0';
        this.ctx.font = '10px monospace';
        
        const columns = Math.floor(this.canvas.width / 10);
        
        for (let i = 0; i < columns; i++) {
            // 随机字符
            const char = this.chars[Math.floor(Math.random() * this.chars.length)];
            
            // 绘制字符
            this.ctx.fillText(char, i * 10, this.drops[i] * 10);
            
            // 重置雨滴位置
            if (this.drops[i] * 10 > this.canvas.height && Math.random() > 0.975) {
                this.drops[i] = 0;
            }
            
            // 雨滴下落
            this.drops[i]++;
        }
        
        setTimeout(() => this.animate(), 35);
    }
}

// 页面加载完成后启动矩阵雨
document.addEventListener('DOMContentLoaded', () => {
    new MatrixRain();
});