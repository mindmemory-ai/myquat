//! Performance Configuration Demo
//!
//! This example demonstrates the unified performance configuration system
//! that controls Rayon, SIMD, and GPU optimizations automatically or manually.

use myquat::{
    performance_config::{global_performance_manager, PerformanceConfig, PerformanceManager},
    Result,
};

fn main() -> Result<()> {
    println!("MyQuat 性能配置系统演示");
    println!("{}", "=".repeat(50));

    // Demo 1: 默认配置和硬件检测
    demonstrate_default_config()?;

    // Demo 2: 自定义配置
    demonstrate_custom_config()?;

    // Demo 3: 配置文件加载
    demonstrate_config_file()?;

    // Demo 4: 动态配置调整
    demonstrate_dynamic_config()?;

    // Demo 5: 性能决策演示
    demonstrate_performance_decisions()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_default_config() -> Result<()> {
    println!("\n1. 默认配置和硬件检测");
    println!("{}", "-".repeat(30));

    let manager = PerformanceManager::new();
    let report = manager.create_performance_report();

    println!("{}", report);

    Ok(())
}

fn demonstrate_custom_config() -> Result<()> {
    println!("\n2. 自定义配置");
    println!("{}", "-".repeat(30));

    // 创建自定义配置
    let mut config = PerformanceConfig::default();

    // 调整并行计算设置
    config.parallel.min_qubits_for_parallel = 6; // 更积极的并行化
    config.parallel.num_threads = Some(4); // 固定4线程

    // 调整 SIMD 设置
    config.simd.min_state_size = 64; // 更小的阈值 (6 qubits)
    config.simd.force_enable = true; // 强制启用

    // 调整 GPU 设置
    config.gpu.min_qubits_for_gpu = 10; // 更积极的GPU使用

    let manager = PerformanceManager::with_config(config);
    let report = manager.create_performance_report();

    println!("自定义配置报告:");
    println!("{}", report);

    // 测试决策逻辑
    println!("\n决策测试:");
    for qubits in [4, 6, 8, 10, 12, 16] {
        let state_size = 1 << qubits;
        println!("  {} 量子比特:", qubits);
        println!(
            "    并行计算: {}",
            if manager.should_use_parallel(qubits) {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "    SIMD优化: {}",
            if manager.should_use_simd(state_size) {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "    GPU加速: {}",
            if manager.should_use_gpu(qubits) {
                "✅"
            } else {
                "❌"
            }
        );
    }

    Ok(())
}

fn demonstrate_config_file() -> Result<()> {
    println!("\n3. 配置文件加载");
    println!("{}", "-".repeat(30));

    let manager = PerformanceManager::new();

    // 尝试加载配置文件
    match manager.load_from_file("myquat_config.toml") {
        Ok(_) => {
            println!("✅ 配置文件加载成功");
            let report = manager.create_performance_report();
            println!("{}", report);
        }
        Err(e) => {
            println!("⚠️  配置文件加载失败: {}", e);
            println!("使用默认配置");

            // 创建示例配置文件
            if let Err(e) = manager.save_to_file("myquat_config_example.toml") {
                println!("❌ 无法创建示例配置文件: {}", e);
            } else {
                println!("✅ 已创建示例配置文件: myquat_config_example.toml");
            }
        }
    }

    Ok(())
}

fn demonstrate_dynamic_config() -> Result<()> {
    println!("\n4. 动态配置调整");
    println!("{}", "-".repeat(30));

    let manager = PerformanceManager::new();

    println!("初始配置:");
    let initial_config = manager.get_config();
    println!("  并行计算: {}", initial_config.parallel.enabled);
    println!("  SIMD优化: {}", initial_config.simd.enabled);
    println!("  GPU加速: {}", initial_config.gpu.enabled);

    // 动态调整配置
    manager.update_config(|config| {
        config.parallel.enabled = false; // 禁用并行计算
        config.simd.enabled = false; // 禁用 SIMD
        config.gpu.enabled = false; // 禁用 GPU
    })?;

    println!("\n调整后配置:");
    let updated_config = manager.get_config();
    println!("  并行计算: {}", updated_config.parallel.enabled);
    println!("  SIMD优化: {}", updated_config.simd.enabled);
    println!("  GPU加速: {}", updated_config.gpu.enabled);

    // 测试决策变化
    println!("\n决策变化 (12 量子比特):");
    println!(
        "  并行计算: {}",
        if manager.should_use_parallel(12) {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  SIMD优化: {}",
        if manager.should_use_simd(4096) {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  GPU加速: {}",
        if manager.should_use_gpu(12) {
            "✅"
        } else {
            "❌"
        }
    );

    // 恢复配置
    manager.update_config(|config| {
        config.parallel.enabled = true;
        config.simd.enabled = true;
        config.gpu.enabled = true;
    })?;

    println!("\n恢复后决策 (12 量子比特):");
    println!(
        "  并行计算: {}",
        if manager.should_use_parallel(12) {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  SIMD优化: {}",
        if manager.should_use_simd(4096) {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  GPU加速: {}",
        if manager.should_use_gpu(12) {
            "✅"
        } else {
            "❌"
        }
    );

    Ok(())
}

fn demonstrate_performance_decisions() -> Result<()> {
    println!("\n5. 性能决策演示");
    println!("{}", "-".repeat(30));

    // 使用全局配置管理器
    let manager = global_performance_manager();

    println!("不同问题规模的优化策略:");
    println!("量子比特 | 状态大小 | 并行 | SIMD | GPU | 推荐策略");
    println!("{}", "-".repeat(60));

    for qubits in [4, 6, 8, 10, 12, 14, 16] {
        let state_size = 1 << qubits;
        let parallel = if manager.should_use_parallel(qubits) {
            "✅"
        } else {
            "❌"
        };
        let simd = if manager.should_use_simd(state_size) {
            "✅"
        } else {
            "❌"
        };
        let gpu = if manager.should_use_gpu(qubits) {
            "✅"
        } else {
            "❌"
        };

        let strategy = match (
            manager.should_use_parallel(qubits),
            manager.should_use_simd(state_size),
            manager.should_use_gpu(qubits),
        ) {
            (false, false, false) => "标量CPU",
            (true, false, false) => "多核CPU",
            (false, true, false) => "SIMD CPU",
            (true, true, false) => "并行+SIMD",
            (false, false, true) => "GPU",
            (true, false, true) => "GPU+并行",
            (false, true, true) => "GPU+SIMD",
            (true, true, true) => "全优化",
        };

        println!(
            "   {:2}    | {:8} |  {}  |  {}  | {}  | {}",
            qubits, state_size, parallel, simd, gpu, strategy
        );
    }

    // 性能建议
    let report = manager.create_performance_report();
    if !report.recommendations.is_empty() {
        println!("\n💡 性能优化建议:");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
    } else {
        println!("\n✅ 当前配置已优化，无需调整");
    }

    // 线程数建议
    println!("\n🧵 线程配置:");
    println!("  推荐线程数: {}", manager.get_num_threads());
    println!(
        "  系统核心数: {}",
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    );

    Ok(())
}
