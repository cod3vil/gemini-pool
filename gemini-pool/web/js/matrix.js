// 矩阵雨效果
class MatrixRain {
    constructor() {
        this.canvas = document.getElementById('matrix');
        this.ctx = this.canvas.getContext('2d');
        this.drops = [];
        this.chars = '01アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン';
        
        this.init();
        this.animate();
    }
    
    init() {
        this.resizeCanvas();
        window.addEventListener('resize', () => this.resizeCanvas());
        
        // 初始化雨滴
        const columns = Math.floor(this.canvas.width / 20);
        for (let i = 0; i < columns; i++) {
            this.drops[i] = Math.random() * this.canvas.height;
        }
    }
    
    resizeCanvas() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }
    
    animate() {
        // 半透明背景，创建拖尾效果
        this.ctx.fillStyle = 'rgba(10, 10, 10, 0.05)';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // 设置文字样式
        this.ctx.fillStyle = '#00f5ff';
        this.ctx.font = '15px monospace';
        
        const columns = Math.floor(this.canvas.width / 20);
        
        for (let i = 0; i < columns; i++) {
            // 随机字符
            const char = this.chars[Math.floor(Math.random() * this.chars.length)];
            
            // 绘制字符
            this.ctx.fillText(char, i * 20, this.drops[i]);
            
            // 重置雨滴位置
            if (this.drops[i] > this.canvas.height && Math.random() > 0.975) {
                this.drops[i] = 0;
            }
            
            // 雨滴下落
            this.drops[i] += 20;
        }
        
        requestAnimationFrame(() => this.animate());
    }
}

// 页面加载完成后启动矩阵雨
document.addEventListener('DOMContentLoaded', () => {
    new MatrixRain();
});