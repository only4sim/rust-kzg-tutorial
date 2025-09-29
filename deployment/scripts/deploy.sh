#!/bin/bash

# KZG 生产环境部署脚本
# 支持 Docker 和 Kubernetes 部署

set -euo pipefail

# 脚本配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOYMENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 默认配置
ENVIRONMENT="${ENVIRONMENT:-production}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
NAMESPACE="${NAMESPACE:-kzg-production}"
KUBECTL_TIMEOUT="${KUBECTL_TIMEOUT:-300s}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示使用说明
show_usage() {
    cat << EOF
KZG 生产环境部署脚本

使用方法:
    $0 [COMMAND] [OPTIONS]

命令:
    build           构建 Docker 镜像
    push            推送镜像到仓库
    deploy          部署到 Kubernetes
    rollback        回滚到上一版本
    status          查看部署状态
    logs            查看服务日志
    clean           清理资源

选项:
    -e, --environment ENV    环境名称 (default: production)
    -t, --tag TAG           镜像标签 (default: latest)
    -n, --namespace NS      Kubernetes 命名空间 (default: kzg-production)
    -h, --help              显示此帮助信息

环境变量:
    DOCKER_REGISTRY         Docker 镜像仓库地址
    KUBECONFIG             Kubernetes 配置文件路径
    IMAGE_TAG              镜像标签
    NAMESPACE              部署命名空间

示例:
    # 构建并部署
    $0 build
    $0 deploy

    # 指定标签部署
    IMAGE_TAG=v1.2.3 $0 deploy

    # 查看状态
    $0 status

    # 查看日志
    $0 logs
EOF
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            build|push|deploy|rollback|status|logs|clean)
                COMMAND="$1"
                shift
                ;;
            *)
                log_error "未知参数: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# 检查依赖
check_dependencies() {
    local deps=("docker" "kubectl")
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log_error "缺少依赖: $dep"
            exit 1
        fi
    done
    
    # 检查 Docker 守护进程
    if ! docker info &> /dev/null; then
        log_error "Docker 守护进程未运行"
        exit 1
    fi
    
    # 检查 Kubernetes 连接
    if ! kubectl cluster-info &> /dev/null; then
        log_warning "无法连接到 Kubernetes 集群"
    fi
}

# 构建 Docker 镜像
build_image() {
    log_info "开始构建 Docker 镜像..."
    
    local image_name="kzg-service:${IMAGE_TAG}"
    local dockerfile="$DEPLOYMENT_ROOT/docker/Dockerfile.production"
    
    if [[ ! -f "$dockerfile" ]]; then
        log_error "Dockerfile 不存在: $dockerfile"
        exit 1
    fi
    
    log_info "构建镜像: $image_name"
    docker build \
        -f "$dockerfile" \
        -t "$image_name" \
        --build-arg BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
        --build-arg VERSION="$IMAGE_TAG" \
        --build-arg VCS_REF="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" \
        "$PROJECT_ROOT"
    
    # 添加 latest 标签
    if [[ "$IMAGE_TAG" != "latest" ]]; then
        docker tag "$image_name" "kzg-service:latest"
    fi
    
    log_success "镜像构建完成: $image_name"
}

# 推送镜像
push_image() {
    log_info "推送 Docker 镜像..."
    
    if [[ -z "${DOCKER_REGISTRY:-}" ]]; then
        log_warning "未设置 DOCKER_REGISTRY 环境变量，跳过推送"
        return
    fi
    
    local image_name="$DOCKER_REGISTRY/kzg-service:${IMAGE_TAG}"
    
    # 标记镜像
    docker tag "kzg-service:${IMAGE_TAG}" "$image_name"
    
    # 推送镜像
    log_info "推送到: $image_name"
    docker push "$image_name"
    
    log_success "镜像推送完成"
}

# 部署到 Kubernetes
deploy_k8s() {
    log_info "开始部署到 Kubernetes..."
    
    # 创建命名空间
    log_info "创建命名空间: $NAMESPACE"
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # 应用配置
    log_info "应用 Kubernetes 配置..."
    
    # 应用 ConfigMap
    if [[ -f "$DEPLOYMENT_ROOT/kubernetes/configmap.yaml" ]]; then
        kubectl apply -f "$DEPLOYMENT_ROOT/kubernetes/configmap.yaml" -n "$NAMESPACE"
    fi
    
    # 应用 Secret
    if [[ -f "$DEPLOYMENT_ROOT/kubernetes/secret.yaml" ]]; then
        kubectl apply -f "$DEPLOYMENT_ROOT/kubernetes/secret.yaml" -n "$NAMESPACE"
    fi
    
    # 应用服务配置
    kubectl apply -f "$DEPLOYMENT_ROOT/kubernetes/service.yaml" -n "$NAMESPACE"
    
    # 更新 Deployment 中的镜像标签
    local deployment_file="$DEPLOYMENT_ROOT/kubernetes/deployment.yaml"
    local temp_deployment="/tmp/deployment-${IMAGE_TAG}.yaml"
    
    sed "s|image: kzg-service:.*|image: kzg-service:${IMAGE_TAG}|g" \
        "$deployment_file" > "$temp_deployment"
    
    # 应用 Deployment
    kubectl apply -f "$temp_deployment" -n "$NAMESPACE"
    rm -f "$temp_deployment"
    
    # 等待部署完成
    log_info "等待部署完成..."
    kubectl rollout status deployment/kzg-service -n "$NAMESPACE" --timeout="$KUBECTL_TIMEOUT"
    
    # 应用其他资源
    for file in "$DEPLOYMENT_ROOT/kubernetes"/*.yaml; do
        if [[ -f "$file" ]] && [[ "$file" != *"deployment.yaml" ]] && [[ "$file" != *"service.yaml" ]]; then
            kubectl apply -f "$file" -n "$NAMESPACE"
        fi
    done
    
    log_success "部署完成！"
    
    # 显示部署状态
    show_status
}

# 回滚部署
rollback_deployment() {
    log_info "回滚部署..."
    
    kubectl rollout undo deployment/kzg-service -n "$NAMESPACE"
    kubectl rollout status deployment/kzg-service -n "$NAMESPACE" --timeout="$KUBECTL_TIMEOUT"
    
    log_success "回滚完成！"
}

# 显示部署状态
show_status() {
    log_info "部署状态:"
    
    echo
    echo "=== Pods ==="
    kubectl get pods -n "$NAMESPACE" -l app=kzg-service -o wide
    
    echo
    echo "=== Services ==="
    kubectl get services -n "$NAMESPACE" -l app=kzg-service
    
    echo
    echo "=== Deployment ==="
    kubectl get deployment kzg-service -n "$NAMESPACE"
    
    echo
    echo "=== Events ==="
    kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' | tail -10
}

# 显示日志
show_logs() {
    log_info "显示服务日志..."
    
    local follow_flag=""
    if [[ "${1:-}" == "-f" ]] || [[ "${1:-}" == "--follow" ]]; then
        follow_flag="-f"
    fi
    
    kubectl logs -l app=kzg-service -n "$NAMESPACE" $follow_flag --tail=100
}

# 清理资源
clean_resources() {
    log_warning "清理 Kubernetes 资源..."
    
    read -p "确定要删除命名空间 '$NAMESPACE' 中的所有资源吗? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "取消清理操作"
        return
    fi
    
    kubectl delete namespace "$NAMESPACE" --ignore-not-found=true
    
    log_success "资源清理完成"
}

# 健康检查
health_check() {
    log_info "执行健康检查..."
    
    local pod_name
    pod_name=$(kubectl get pods -n "$NAMESPACE" -l app=kzg-service -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    
    if [[ -z "$pod_name" ]]; then
        log_error "未找到运行中的 Pod"
        return 1
    fi
    
    # 检查健康状态
    local health_url="http://localhost:8080/health"
    kubectl port-forward -n "$NAMESPACE" "pod/$pod_name" 8080:8080 &
    local port_forward_pid=$!
    
    sleep 3
    
    if curl -f "$health_url" >/dev/null 2>&1; then
        log_success "健康检查通过"
        kill $port_forward_pid
        return 0
    else
        log_error "健康检查失败"
        kill $port_forward_pid
        return 1
    fi
}

# 主函数
main() {
    log_info "KZG 生产环境部署脚本"
    log_info "环境: $ENVIRONMENT, 镜像标签: $IMAGE_TAG, 命名空间: $NAMESPACE"
    
    check_dependencies
    
    case "${COMMAND:-}" in
        build)
            build_image
            ;;
        push)
            push_image
            ;;
        deploy)
            build_image
            deploy_k8s
            ;;
        rollback)
            rollback_deployment
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs "$@"
            ;;
        clean)
            clean_resources
            ;;
        "")
            log_error "请指定命令"
            show_usage
            exit 1
            ;;
        *)
            log_error "未知命令: ${COMMAND:-}"
            show_usage
            exit 1
            ;;
    esac
}

# 解析参数并执行
COMMAND=""
parse_args "$@"
main "$@"