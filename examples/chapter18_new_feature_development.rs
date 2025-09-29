// 第18章: 新特性开发指南 - 示例代码
// 
// 本示例演示如何为 rust-kzg 项目开发新功能的完整流程，
// 包括需求分析、技术设计、代码实现、测试和文档。

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::process::Command;
use serde::{Deserialize, Serialize};

// ============================================================================
// 18.1 需求分析和设计
// ============================================================================

/// 用户需求分析器
/// 
/// 系统化收集、分析和管理功能需求，为新特性开发提供指导
#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementAnalyzer {
    requirements: Vec<Requirement>,
    stakeholders: Vec<Stakeholder>,
    use_cases: Vec<UseCase>,
}

/// 需求定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub category: Category,
    pub stakeholder: String,
    pub acceptance_criteria: Vec<String>,
}

/// 需求优先级
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// 需求类别
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Category {
    Performance,
    Security,
    Usability,
    Compatibility,
    Documentation,
}

/// 利益相关者
#[derive(Debug, Serialize, Deserialize)]
pub struct Stakeholder {
    pub name: String,
    pub role: String,
    pub expertise_areas: Vec<String>,
    pub contact: String,
}

/// 用例定义
#[derive(Debug, Serialize, Deserialize)]
pub struct UseCase {
    pub id: String,
    pub title: String,
    pub actor: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<String>,
    pub postconditions: Vec<String>,
    pub alternative_flows: Vec<String>,
}

impl RequirementAnalyzer {
    /// 创建新的需求分析器
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
            stakeholders: Vec::new(),
            use_cases: Vec::new(),
        }
    }
    
    /// 添加利益相关者
    pub fn add_stakeholder(&mut self, stakeholder: Stakeholder) {
        println!("👥 添加利益相关者: {} ({})", stakeholder.name, stakeholder.role);
        self.stakeholders.push(stakeholder);
    }
    
    /// 添加需求
    pub fn add_requirement(&mut self, req: Requirement) {
        println!("📝 添加需求: {} - {} (优先级: {:?})", req.id, req.title, req.priority);
        self.requirements.push(req);
    }
    
    /// 添加用例
    pub fn add_use_case(&mut self, use_case: UseCase) {
        println!("📋 添加用例: {} - {}", use_case.id, use_case.title);
        self.use_cases.push(use_case);
    }
    
    /// 分析需求优先级分布
    pub fn analyze_priorities(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        
        for req in &self.requirements {
            let priority = format!("{:?}", req.priority);
            *distribution.entry(priority).or_insert(0) += 1;
        }
        
        println!("\n📊 需求优先级分布:");
        for (priority, count) in &distribution {
            println!("   {} 优先级: {} 个需求", priority, count);
        }
        
        distribution
    }
    
    /// 分析需求类别分布
    pub fn analyze_categories(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        
        for req in &self.requirements {
            let category = format!("{:?}", req.category);
            *distribution.entry(category).or_insert(0) += 1;
        }
        
        println!("\n📊 需求类别分布:");
        for (category, count) in &distribution {
            println!("   {} 类别: {} 个需求", category, count);
        }
        
        distribution
    }
    
    /// 生成需求分析报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# 需求分析报告\n\n");
        
        // 概览信息
        report.push_str("## 📊 项目概览\n\n");
        report.push_str(&format!("- 📋 总需求数: {}\n", self.requirements.len()));
        report.push_str(&format!("- 👥 利益相关者: {}\n", self.stakeholders.len()));
        report.push_str(&format!("- 🎯 用例数: {}\n", self.use_cases.len()));
        report.push_str(&format!("- 📅 生成时间: {}\n\n", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
        
        // 优先级分布
        let priorities = self.analyze_priorities();
        report.push_str("## 📊 需求优先级分布\n\n");
        for (priority, count) in &priorities {
            let percentage = (*count as f64 / self.requirements.len() as f64) * 100.0;
            report.push_str(&format!("- **{}**: {} 个需求 ({:.1}%)\n", priority, count, percentage));
        }
        report.push('\n');
        
        // 详细需求
        report.push_str("## 📋 需求详情\n\n");
        for req in &self.requirements {
            report.push_str(&format!(
                "### {} - {}\n\n",
                req.id, req.title
            ));
            report.push_str(&format!("**优先级**: {:?}  \n", req.priority));
            report.push_str(&format!("**类别**: {:?}  \n", req.category));
            report.push_str(&format!("**负责人**: {}  \n\n", req.stakeholder));
            report.push_str(&format!("**描述**: {}  \n\n", req.description));
            
            if !req.acceptance_criteria.is_empty() {
                report.push_str("**验收标准**:\n");
                for criterion in &req.acceptance_criteria {
                    report.push_str(&format!("- {}\n", criterion));
                }
                report.push('\n');
            }
            
            report.push_str("---\n\n");
        }
        
        report
    }
}

// ============================================================================
// 技术可行性分析
// ============================================================================

/// 技术可行性评估器
pub struct FeasibilityAnalyzer {
    criteria: Vec<FeasibilityCriteria>,
}

/// 可行性评估标准
#[derive(Debug, Clone)]
pub struct FeasibilityCriteria {
    pub name: String,
    pub weight: f64,
    pub score: f64,
    pub rationale: String,
}

/// 可行性评估结果
#[derive(Debug)]
pub struct FeasibilityResult {
    pub overall_score: f64,
    pub recommendation: Recommendation,
    pub risks: Vec<Risk>,
    pub mitigation_strategies: Vec<String>,
    pub estimated_effort: EstimatedEffort,
}

/// 推荐等级
#[derive(Debug)]
pub enum Recommendation {
    HighlyRecommended,  // ≥ 8.0
    Recommended,        // ≥ 6.0
    Conditional,        // ≥ 4.0
    NotRecommended,     // < 4.0
}

/// 风险定义
#[derive(Debug)]
pub struct Risk {
    pub description: String,
    pub probability: f64,    // 0.0 - 1.0
    pub impact: f64,         // 0.0 - 1.0
    pub severity: RiskSeverity,
}

/// 风险严重程度
#[derive(Debug)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 工作量估算
#[derive(Debug)]
pub struct EstimatedEffort {
    pub development_days: u32,
    pub testing_days: u32,
    pub documentation_days: u32,
    pub total_days: u32,
}

impl FeasibilityAnalyzer {
    /// 创建标准化的可行性分析器
    pub fn new() -> Self {
        Self {
            criteria: vec![
                FeasibilityCriteria {
                    name: "技术复杂度".to_string(),
                    weight: 0.25,
                    score: 0.0,
                    rationale: "实现的技术难度和风险".to_string(),
                },
                FeasibilityCriteria {
                    name: "资源需求".to_string(),
                    weight: 0.20,
                    score: 0.0,
                    rationale: "所需的人力和时间资源".to_string(),
                },
                FeasibilityCriteria {
                    name: "时间成本".to_string(),
                    weight: 0.15,
                    score: 0.0,
                    rationale: "开发周期和时间压力".to_string(),
                },
                FeasibilityCriteria {
                    name: "兼容性影响".to_string(),
                    weight: 0.20,
                    score: 0.0,
                    rationale: "对现有系统的兼容性影响".to_string(),
                },
                FeasibilityCriteria {
                    name: "维护成本".to_string(),
                    weight: 0.10,
                    score: 0.0,
                    rationale: "长期维护的复杂度和成本".to_string(),
                },
                FeasibilityCriteria {
                    name: "商业价值".to_string(),
                    weight: 0.10,
                    score: 0.0,
                    rationale: "功能带来的商业价值和用户价值".to_string(),
                },
            ],
        }
    }
    
    /// 设置评估分数
    pub fn set_score(&mut self, criteria_name: &str, score: f64, rationale: &str) -> Result<(), String> {
        if score < 0.0 || score > 10.0 {
            return Err("分数必须在 0.0 到 10.0 之间".to_string());
        }
        
        let criteria = self.criteria.iter_mut()
            .find(|c| c.name == criteria_name)
            .ok_or_else(|| format!("未找到评估标准: {}", criteria_name))?;
        
        criteria.score = score;
        criteria.rationale = rationale.to_string();
        
        println!("📊 设置 {} 评分: {:.1}/10 - {}", criteria_name, score, rationale);
        Ok(())
    }
    
    /// 执行可行性评估
    pub fn evaluate(&self) -> FeasibilityResult {
        let overall_score = self.calculate_weighted_score();
        
        let recommendation = match overall_score {
            score if score >= 8.0 => Recommendation::HighlyRecommended,
            score if score >= 6.0 => Recommendation::Recommended,
            score if score >= 4.0 => Recommendation::Conditional,
            _ => Recommendation::NotRecommended,
        };
        
        let risks = self.identify_risks();
        let mitigation_strategies = self.suggest_mitigations(&risks);
        let estimated_effort = self.estimate_effort(overall_score);
        
        println!("\n🎯 可行性评估完成:");
        println!("   📊 综合得分: {:.1}/10", overall_score);
        println!("   🎯 推荐等级: {:?}", recommendation);
        println!("   ⚠️  识别风险: {} 个", risks.len());
        println!("   📅 预估工期: {} 天", estimated_effort.total_days);
        
        FeasibilityResult {
            overall_score,
            recommendation,
            risks,
            mitigation_strategies,
            estimated_effort,
        }
    }
    
    /// 计算加权总分
    fn calculate_weighted_score(&self) -> f64 {
        let weighted_sum: f64 = self.criteria.iter()
            .map(|c| c.score * c.weight)
            .sum();
        
        // 显示详细评分
        println!("\n📊 详细评分:");
        for criteria in &self.criteria {
            let weighted_score = criteria.score * criteria.weight;
            println!("   {}: {:.1}/10 (权重 {:.0}%) = {:.2}", 
                     criteria.name, criteria.score, criteria.weight * 100.0, weighted_score);
        }
        
        weighted_sum
    }
    
    /// 识别风险因素
    fn identify_risks(&self) -> Vec<Risk> {
        let mut risks = Vec::new();
        
        for criteria in &self.criteria {
            if criteria.score < 5.0 {
                let severity = match criteria.score {
                    score if score < 3.0 => RiskSeverity::Critical,
                    score if score < 4.0 => RiskSeverity::High,
                    score if score < 5.0 => RiskSeverity::Medium,
                    _ => RiskSeverity::Low,
                };
                
                let risk = Risk {
                    description: format!("{}存在挑战: {}", criteria.name, criteria.rationale),
                    probability: (5.0 - criteria.score) / 5.0,
                    impact: criteria.weight,
                    severity,
                };
                
                risks.push(risk);
            }
        }
        
        risks
    }
    
    /// 建议风险缓解策略
    fn suggest_mitigations(&self, risks: &[Risk]) -> Vec<String> {
        let mut strategies = Vec::new();
        
        if !risks.is_empty() {
            strategies.push("制定详细的技术调研和原型验证计划".to_string());
            strategies.push("分阶段实施，降低整体风险".to_string());
            strategies.push("建立回滚机制和应急方案".to_string());
        }
        
        if risks.iter().any(|r| matches!(r.severity, RiskSeverity::High | RiskSeverity::Critical)) {
            strategies.push("安排技术专家进行深度评估".to_string());
            strategies.push("考虑引入外部咨询或技术支持".to_string());
        }
        
        strategies.push("建立定期风险评估和调整机制".to_string());
        
        strategies
    }
    
    /// 估算工作量
    fn estimate_effort(&self, overall_score: f64) -> EstimatedEffort {
        // 基于复杂度和分数估算工作量
        let complexity_factor = (10.0 - overall_score) / 10.0;
        
        let base_development_days = 10;
        let development_days = (base_development_days as f64 * (1.0 + complexity_factor * 2.0)) as u32;
        let testing_days = (development_days as f64 * 0.4) as u32;
        let documentation_days = (development_days as f64 * 0.2) as u32;
        
        EstimatedEffort {
            development_days,
            testing_days,
            documentation_days,
            total_days: development_days + testing_days + documentation_days,
        }
    }
}

// ============================================================================
// 18.2 开发流程管理
// ============================================================================

/// Git 工作流管理器
pub struct GitWorkflowManager {
    repo_path: String,
    current_branch: String,
}

impl GitWorkflowManager {
    /// 创建工作流管理器
    pub fn new(repo_path: String) -> Self {
        let current_branch = Self::get_current_branch(&repo_path)
            .unwrap_or_else(|| "main".to_string());
        
        Self {
            repo_path,
            current_branch,
        }
    }
    
    /// 创建新功能分支
    pub fn create_feature_branch(&mut self, feature_name: &str) -> Result<(), String> {
        let branch_name = format!("feature/{}", feature_name);
        
        println!("🌿 创建新功能分支: {}", branch_name);
        
        // 确保在主分支并拉取最新代码
        self.checkout_main()?;
        self.pull_latest()?;
        
        // 创建并切换到新分支
        self.run_git_command(&["checkout", "-b", &branch_name])?;
        
        self.current_branch = branch_name;
        println!("✅ 功能分支创建成功: {}", self.current_branch);
        
        Ok(())
    }
    
    /// 提交代码更改
    pub fn commit_changes(&self, message: &str, files: Option<Vec<&str>>) -> Result<(), String> {
        println!("💾 提交代码更改: {}", message);
        
        // 添加指定文件或所有更改
        if let Some(file_list) = files {
            for file in file_list {
                self.run_git_command(&["add", file])?;
            }
        } else {
            self.run_git_command(&["add", "."])?;
        }
        
        // 检查是否有更改要提交
        let status = self.run_git_command(&["status", "--porcelain"])?;
        if status.trim().is_empty() {
            return Err("没有可提交的更改".to_string());
        }
        
        // 提交更改
        self.run_git_command(&["commit", "-m", message])?;
        
        println!("✅ 提交成功: {}", message);
        Ok(())
    }
    
    /// 推送分支到远程仓库
    pub fn push_branch(&self) -> Result<(), String> {
        println!("⬆️ 推送分支: {} -> origin", self.current_branch);
        
        self.run_git_command(&["push", "-u", "origin", &self.current_branch])?;
        
        println!("✅ 推送成功");
        Ok(())
    }
    
    /// 创建 Pull Request (需要 GitHub CLI)
    pub fn create_pull_request(&self, title: &str, body: &str, labels: Option<Vec<&str>>) -> Result<(), String> {
        println!("📥 创建 Pull Request: {}", title);
        
        let mut args = vec!["pr", "create", "--title", title, "--body", body];
        
        // 添加标签
        if let Some(label_list) = labels {
            for label in label_list {
                args.push("--label");
                args.push(label);
            }
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("创建 PR 失败: {}", e))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("创建 PR 失败: {}", error));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        println!("✅ Pull Request 创建成功");
        println!("🔗 {}", result.trim());
        
        Ok(())
    }
    
    /// 执行完整的功能开发工作流
    pub fn complete_feature_workflow(
        &mut self,
        feature_name: &str,
        commit_message: &str,
        pr_title: &str,
        pr_body: &str,
    ) -> Result<(), String> {
        println!("🚀 执行完整功能开发工作流: {}", feature_name);
        
        // 1. 创建功能分支
        self.create_feature_branch(feature_name)?;
        
        // 2. 提交更改
        self.commit_changes(commit_message, None)?;
        
        // 3. 推送分支
        self.push_branch()?;
        
        // 4. 创建 Pull Request
        self.create_pull_request(pr_title, pr_body, Some(vec!["enhancement", "needs-review"]))?;
        
        println!("🎉 功能开发工作流完成!");
        Ok(())
    }
    
    fn checkout_main(&self) -> Result<(), String> {
        self.run_git_command(&["checkout", "main"])?;
        Ok(())
    }
    
    fn pull_latest(&self) -> Result<(), String> {
        self.run_git_command(&["pull", "origin", "main"])?;
        Ok(())
    }
    
    fn run_git_command(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("Git 命令执行失败: {}", e))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git 命令失败: {}", error));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    fn get_current_branch(repo_path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(repo_path)
            .output()
            .ok()?;
        
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}

// ============================================================================
// 代码质量检查
// ============================================================================

/// 代码质量检查器
pub struct CodeQualityChecker {
    repo_path: String,
}

/// 质量检查结果
#[derive(Debug)]
pub struct QualityCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub duration: Duration,
}

/// 质量检查报告
#[derive(Debug)]
pub struct QualityReport {
    pub checks: Vec<QualityCheck>,
    pub total_duration: Duration,
}

impl CodeQualityChecker {
    pub fn new(repo_path: String) -> Self {
        Self { repo_path }
    }
    
    /// 运行完整的代码质量检查
    pub fn run_full_check(&self) -> Result<QualityReport, String> {
        let mut checks = Vec::new();
        let start_time = Instant::now();
        
        println!("🔍 开始代码质量检查...");
        
        // 1. 代码格式检查
        checks.push(self.check_formatting());
        
        // 2. Clippy 静态分析  
        checks.push(self.run_clippy());
        
        // 3. 单元测试
        checks.push(self.run_tests());
        
        // 4. 文档检查
        checks.push(self.check_docs());
        
        // 5. 安全审计 (如果有 cargo-audit)
        checks.push(self.security_audit());
        
        let total_duration = start_time.elapsed();
        
        let report = QualityReport {
            checks,
            total_duration,
        };
        
        self.print_summary(&report);
        
        Ok(report)
    }
    
    fn check_formatting(&self) -> QualityCheck {
        let start = Instant::now();
        
        let result = Command::new("cargo")
            .args(&["fmt", "--check"])
            .current_dir(&self.repo_path)
            .output();
        
        let duration = start.elapsed();
        
        match result {
            Ok(output) if output.status.success() => {
                QualityCheck {
                    name: "代码格式".to_string(),
                    passed: true,
                    message: "代码格式符合规范".to_string(),
                    duration,
                }
            }
            Ok(_) => {
                QualityCheck {
                    name: "代码格式".to_string(),
                    passed: false,
                    message: "代码格式不符合规范，请运行 cargo fmt".to_string(),
                    duration,
                }
            }
            Err(e) => {
                QualityCheck {
                    name: "代码格式".to_string(),
                    passed: false,
                    message: format!("格式检查失败: {}", e),
                    duration,
                }
            }
        }
    }
    
    fn run_clippy(&self) -> QualityCheck {
        let start = Instant::now();
        
        let result = Command::new("cargo")
            .args(&["clippy", "--all-targets", "--", "-D", "warnings"])
            .current_dir(&self.repo_path)
            .output();
        
        let duration = start.elapsed();
        
        match result {
            Ok(output) if output.status.success() => {
                QualityCheck {
                    name: "Clippy 检查".to_string(),
                    passed: true,
                    message: "没有发现警告".to_string(),
                    duration,
                }
            }
            Ok(output) => {
                let warnings = String::from_utf8_lossy(&output.stderr);
                QualityCheck {
                    name: "Clippy 检查".to_string(),
                    passed: false,
                    message: format!("发现问题: {}", warnings.chars().take(200).collect::<String>()),
                    duration,
                }
            }
            Err(e) => {
                QualityCheck {
                    name: "Clippy 检查".to_string(),
                    passed: false,
                    message: format!("Clippy 检查失败: {}", e),
                    duration,
                }
            }
        }
    }
    
    fn run_tests(&self) -> QualityCheck {
        let start = Instant::now();
        
        let result = Command::new("cargo")
            .args(&["test", "--quiet"])
            .current_dir(&self.repo_path)
            .output();
        
        let duration = start.elapsed();
        
        match result {
            Ok(output) if output.status.success() => {
                QualityCheck {
                    name: "单元测试".to_string(),
                    passed: true,
                    message: "所有测试通过".to_string(),
                    duration,
                }
            }
            Ok(output) => {
                let errors = String::from_utf8_lossy(&output.stderr);
                QualityCheck {
                    name: "单元测试".to_string(),
                    passed: false,
                    message: format!("测试失败: {}", errors.chars().take(200).collect::<String>()),
                    duration,
                }
            }
            Err(e) => {
                QualityCheck {
                    name: "单元测试".to_string(),
                    passed: false,
                    message: format!("测试运行失败: {}", e),
                    duration,
                }
            }
        }
    }
    
    fn check_docs(&self) -> QualityCheck {
        let start = Instant::now();
        
        let result = Command::new("cargo")
            .args(&["doc", "--no-deps", "--quiet"])
            .current_dir(&self.repo_path)
            .output();
        
        let duration = start.elapsed();
        
        match result {
            Ok(output) if output.status.success() => {
                QualityCheck {
                    name: "文档检查".to_string(),
                    passed: true,
                    message: "文档生成成功".to_string(),
                    duration,
                }
            }
            Ok(output) => {
                let errors = String::from_utf8_lossy(&output.stderr);
                QualityCheck {
                    name: "文档检查".to_string(),
                    passed: false,
                    message: format!("文档生成失败: {}", errors.chars().take(200).collect::<String>()),
                    duration,
                }
            }
            Err(e) => {
                QualityCheck {
                    name: "文档检查".to_string(),
                    passed: false,
                    message: format!("文档检查失败: {}", e),
                    duration,
                }
            }
        }
    }
    
    fn security_audit(&self) -> QualityCheck {
        let start = Instant::now();
        
        // 检查是否安装了 cargo-audit
        let audit_available = Command::new("cargo")
            .args(&["audit", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        let duration = start.elapsed();
        
        if !audit_available {
            return QualityCheck {
                name: "安全审计".to_string(),
                passed: true,
                message: "cargo-audit 未安装，跳过安全检查".to_string(),
                duration,
            };
        }
        
        let result = Command::new("cargo")
            .args(&["audit"])
            .current_dir(&self.repo_path)
            .output();
        
        match result {
            Ok(output) if output.status.success() => {
                QualityCheck {
                    name: "安全审计".to_string(),
                    passed: true,
                    message: "没有发现安全漏洞".to_string(),
                    duration,
                }
            }
            Ok(output) => {
                let warnings = String::from_utf8_lossy(&output.stdout);
                QualityCheck {
                    name: "安全审计".to_string(),
                    passed: false,
                    message: format!("发现安全问题: {}", warnings.chars().take(200).collect::<String>()),
                    duration,
                }
            }
            Err(e) => {
                QualityCheck {
                    name: "安全审计".to_string(),
                    passed: false,
                    message: format!("安全审计失败: {}", e),
                    duration,
                }
            }
        }
    }
    
    fn print_summary(&self, report: &QualityReport) {
        let passed = report.checks.iter().filter(|c| c.passed).count();
        let total = report.checks.len();
        
        println!("\n📊 代码质量检查结果:");
        println!("   ✅ 通过: {}/{}", passed, total);
        println!("   ⏱️  总耗时: {:.2}s", report.total_duration.as_secs_f64());
        
        for check in &report.checks {
            let status = if check.passed { "✅" } else { "❌" };
            println!("   {} {}: {} ({:.1}s)", 
                     status, 
                     check.name, 
                     check.message, 
                     check.duration.as_secs_f64());
        }
    }
}

impl QualityReport {
    /// 检查是否所有质量检查都通过
    pub fn is_passing(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }
    
    /// 获取通过率
    pub fn success_rate(&self) -> f64 {
        if self.checks.is_empty() {
            return 100.0;
        }
        
        let passed = self.checks.iter().filter(|c| c.passed).count();
        (passed as f64 / self.checks.len() as f64) * 100.0
    }
}

// ============================================================================
// 18.3 测试框架管理
// ============================================================================

/// 测试套件管理器
pub struct TestSuiteManager {
    test_suites: Vec<TestSuite>,
}

/// 测试套件
#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub category: TestCategory,
    pub tests: Vec<TestCase>,
}

/// 测试类别
#[derive(Debug, PartialEq)]
pub enum TestCategory {
    Unit,
    Integration,
    Performance,
    Security,
    Compatibility,
}

/// 测试用例
#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub test_fn: fn() -> Result<(), String>,
    pub timeout: Duration,
}

/// 测试结果
#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
}

/// 测试报告
#[derive(Debug)]
pub struct TestReport {
    pub results: Vec<TestResult>,
    pub total_duration: Duration,
}

impl TestSuiteManager {
    pub fn new() -> Self {
        Self {
            test_suites: Vec::new(),
        }
    }
    
    /// 添加测试套件
    pub fn add_suite(&mut self, suite: TestSuite) {
        println!("📝 添加测试套件: {} ({:?})", suite.name, suite.category);
        self.test_suites.push(suite);
    }
    
    /// 运行所有测试
    pub fn run_all_tests(&self) -> TestReport {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        println!("🧪 开始运行测试套件...");
        
        for suite in &self.test_suites {
            println!("📋 运行测试套件: {} ({:?})", suite.name, suite.category);
            
            for test_case in &suite.tests {
                let result = self.run_test_case(test_case);
                results.push(result);
            }
        }
        
        let total_duration = start_time.elapsed();
        
        TestReport {
            results,
            total_duration,
        }
    }
    
    /// 运行指定类别的测试
    pub fn run_category_tests(&self, category: TestCategory) -> TestReport {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        println!("🎯 运行 {:?} 类别测试", category);
        
        for suite in &self.test_suites {
            if suite.category == category {
                for test_case in &suite.tests {
                    let result = self.run_test_case(test_case);
                    results.push(result);
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        
        TestReport {
            results,
            total_duration,
        }
    }
    
    fn run_test_case(&self, test_case: &TestCase) -> TestResult {
        println!("  🔬 运行测试: {}", test_case.name);
        
        let start_time = Instant::now();
        
        let result = std::panic::catch_unwind(|| {
            (test_case.test_fn)()
        });
        
        let duration = start_time.elapsed();
        
        let test_result = match result {
            Ok(Ok(_)) => TestResult {
                test_name: test_case.name.clone(),
                passed: true,
                duration,
                error_message: None,
            },
            Ok(Err(e)) => TestResult {
                test_name: test_case.name.clone(),
                passed: false,
                duration,
                error_message: Some(e),
            },
            Err(_) => TestResult {
                test_name: test_case.name.clone(),
                passed: false,
                duration,
                error_message: Some("测试恐慌".to_string()),
            },
        };
        
        let status = if test_result.passed { "✅" } else { "❌" };
        println!("    {} {} ({:.2}ms)", 
                 status, 
                 test_result.test_name,
                 test_result.duration.as_millis());
        
        if let Some(ref error) = test_result.error_message {
            println!("      错误: {}", error);
        }
        
        test_result
    }
}

impl TestReport {
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }
    
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }
    
    pub fn total_count(&self) -> usize {
        self.results.len()
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_count() == 0 {
            return 100.0;
        }
        (self.passed_count() as f64 / self.total_count() as f64) * 100.0
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 测试执行报告:");
        println!("   ✅ 通过: {}", self.passed_count());
        println!("   ❌ 失败: {}", self.failed_count());
        println!("   📊 总计: {}", self.total_count());
        println!("   📈 成功率: {:.1}%", self.success_rate());
        println!("   ⏱️  总耗时: {:.2}s", self.total_duration.as_secs_f64());
        
        if self.failed_count() > 0 {
            println!("\n❌ 失败的测试:");
            for result in &self.results {
                if !result.passed {
                    println!("   - {}: {}", 
                             result.test_name,
                             result.error_message.as_ref().unwrap_or(&"未知错误".to_string()));
                }
            }
        }
    }
}

// ============================================================================
// 演示函数
// ============================================================================

/// 演示完整的新特性开发流程
pub fn demo_feature_development_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 演示第18章: 新特性开发指南");
    println!("{}", "=".repeat(60));
    
    // 1. 需求分析
    println!("\n📋 1. 需求分析阶段");
    
    let mut analyzer = RequirementAnalyzer::new();
    
    // 添加利益相关者
    analyzer.add_stakeholder(Stakeholder {
        name: "以太坊验证节点运维团队".to_string(),
        role: "主要用户".to_string(),
        expertise_areas: vec!["区块链".to_string(), "验证".to_string()],
        contact: "validators@ethereum.org".to_string(),
    });
    
    // 添加需求
    analyzer.add_requirement(Requirement {
        id: "REQ-001".to_string(),
        title: "批量KZG证明验证".to_string(),
        description: "实现同时验证多个KZG证明的功能，提升验证效率".to_string(),
        priority: Priority::High,
        category: Category::Performance,
        stakeholder: "以太坊验证节点运维团队".to_string(),
        acceptance_criteria: vec![
            "批量验证比单个验证快至少3倍".to_string(),
            "支持最多1000个证明的批量验证".to_string(),
            "保持与单个验证相同的安全性".to_string(),
            "提供清晰的API和文档".to_string(),
        ],
    });
    
    analyzer.add_requirement(Requirement {
        id: "REQ-002".to_string(),
        title: "向后兼容性保证".to_string(),
        description: "新功能不能影响现有API的使用".to_string(),
        priority: Priority::Critical,
        category: Category::Compatibility,
        stakeholder: "现有用户".to_string(),
        acceptance_criteria: vec![
            "现有API保持不变".to_string(),
            "现有测试全部通过".to_string(),
        ],
    });
    
    analyzer.analyze_priorities();
    analyzer.analyze_categories();
    
    // 2. 可行性分析
    println!("\n🔍 2. 技术可行性评估");
    
    let mut feasibility = FeasibilityAnalyzer::new();
    
    feasibility.set_score("技术复杂度", 7.0, "需要实现随机线性组合算法，中等复杂度")?;
    feasibility.set_score("资源需求", 8.0, "需要1-2名开发人员，2-3周时间")?;
    feasibility.set_score("时间成本", 7.5, "开发周期适中，风险可控")?;
    feasibility.set_score("兼容性影响", 9.0, "新增API，不影响现有功能")?;
    feasibility.set_score("维护成本", 8.0, "代码结构清晰，维护成本低")?;
    feasibility.set_score("商业价值", 9.0, "显著提升性能，用户价值高")?;
    
    let result = feasibility.evaluate();
    
    match result.recommendation {
        Recommendation::HighlyRecommended | Recommendation::Recommended => {
            println!("✅ 可行性评估通过，建议实施!");
        }
        _ => {
            println!("❌ 可行性评估未通过，需要重新评估");
            return Ok(());
        }
    }
    
    // 3. 代码质量检查演示
    println!("\n🔍 3. 代码质量检查演示");
    
    let checker = CodeQualityChecker::new(".".to_string());
    let quality_report = checker.run_full_check()?;
    
    if !quality_report.is_passing() {
        println!("⚠️ 代码质量检查未完全通过，成功率: {:.1}%", quality_report.success_rate());
    }
    
    // 4. 测试框架演示
    println!("\n🧪 4. 测试框架演示");
    
    let mut test_manager = TestSuiteManager::new();
    
    // 添加单元测试套件
    test_manager.add_suite(TestSuite {
        name: "批量验证单元测试".to_string(),
        category: TestCategory::Unit,
        tests: vec![
            TestCase {
                name: "test_empty_batch".to_string(),
                description: "测试空批次处理".to_string(),
                test_fn: || Ok(()),
                timeout: Duration::from_secs(1),
            },
            TestCase {
                name: "test_single_proof_batch".to_string(),
                description: "测试单个证明批量验证".to_string(),
                test_fn: || Ok(()),
                timeout: Duration::from_secs(1),
            },
            TestCase {
                name: "test_multiple_proofs_batch".to_string(),
                description: "测试多个证明批量验证".to_string(),
                test_fn: || Ok(()),
                timeout: Duration::from_secs(5),
            },
        ],
    });
    
    // 添加性能测试套件
    test_manager.add_suite(TestSuite {
        name: "批量验证性能测试".to_string(),
        category: TestCategory::Performance,
        tests: vec![
            TestCase {
                name: "benchmark_batch_vs_individual".to_string(),
                description: "对比批量验证和单个验证性能".to_string(),
                test_fn: || {
                    // 模拟性能测试
                    std::thread::sleep(Duration::from_millis(100));
                    Ok(())
                },
                timeout: Duration::from_secs(10),
            },
        ],
    });
    
    let test_report = test_manager.run_all_tests();
    test_report.print_summary();
    
    println!("\n🎉 新特性开发流程演示完成!");
    println!("📊 总结:");
    println!("   ✅ 需求分析: {} 个需求已分析", analyzer.requirements.len());
    println!("   ✅ 可行性评估: {:.1}/10 ({:?})", result.overall_score, result.recommendation);
    println!("   ✅ 质量检查: {:.1}% 通过率", quality_report.success_rate());
    println!("   ✅ 测试验证: {:.1}% 成功率", test_report.success_rate());
    
    Ok(())
}

/// 演示实际的批量验证功能实现概念
pub fn demo_batch_verification_concept() {
    println!("\n💡 批量验证功能实现概念演示");
    println!("{}", "=".repeat(50));
    
    // 模拟批量验证的核心思想
    println!("📐 核心算法思想:");
    println!("   1. 随机线性组合: ∑(rᵢ * proofᵢ) where rᵢ 为随机数");
    println!("   2. 单次配对操作: e(∑commitment, g2) ?= e(∑proof, challenge)");
    println!("   3. 复杂度降低: O(n) → O(log n) 对于验证部分");
    
    println!("\n⚡ 性能优势:");
    println!("   - 单个验证: n 次配对运算");
    println!("   - 批量验证: 1 次配对运算 + n 次椭圆曲线运算");
    println!("   - 理论加速: 3-10x (取决于批量大小)");
    
    println!("\n🔒 安全性保证:");
    println!("   - 使用密码学安全的随机数生成器");
    println!("   - 恶意证明无法通过概率可忽略");
    println!("   - 保持与单个验证相同的安全级别");
}

// 测试函数
fn test_basic_functionality() -> Result<(), String> {
    // 基本功能测试
    Ok(())
}

fn test_error_handling() -> Result<(), String> {
    // 错误处理测试 
    Err("模拟错误".to_string())
}

fn test_performance_benchmark() -> Result<(), String> {
    // 性能基准测试
    std::thread::sleep(Duration::from_millis(50));
    Ok(())
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎓 第18章: 新特性开发指南 - 完整演示");
    println!("{}", "=".repeat(80));
    
    // 执行完整的开发流程演示
    demo_feature_development_workflow()?;
    
    // 演示批量验证功能概念
    demo_batch_verification_concept();
    
    println!("\n📚 学习总结:");
    println!("通过本章学习，你已经掌握了:");
    println!("✅ 系统化的需求分析方法");
    println!("✅ 科学的技术可行性评估");
    println!("✅ 标准化的代码开发流程");
    println!("✅ 完善的测试策略和实施");
    println!("✅ 高质量的文档编写规范");
    println!("✅ 有效的社区协作技巧");
    
    println!("\n🎯 实际应用建议:");
    println!("1. 在实际项目中应用这些方法和工具");
    println!("2. 根据项目特点调整流程和标准"); 
    println!("3. 持续改进开发流程和质量标准");
    println!("4. 积极参与开源社区贡献代码");
    
    Ok(())
}