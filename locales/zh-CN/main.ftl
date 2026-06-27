### zh-CN.ftl
### 简体中文翻译，对应书签管理系统模板中的所有界面文案。

## 页面 <title> 区块

page-title-home = 首页
page-title-login = 登录
page-title-changepwd = 修改密码
page-title-bookmark-add = 添加书签
page-title-bookmark-edit = 编辑书签

## 通用品牌 / 导航文案（sidebar.html、home.html、login.html）

brand-name = 书签库
brand-tagline = archive

## login.html

login-heading = 书签库
login-subheading = sign in
login-username-label = 用户名
login-username-placeholder = 用户名
login-password-label = 密码
login-password-placeholder = 密码
login-submit = 登录

## change_password.html

changepwd-heading = 修改密码
changepwd-subheading = change password
changepwd-current-password-label = 当前密码
changepwd-current-password-placeholder = 当前密码
changepwd-new-password-label = 新密码
changepwd-new-password-placeholder = 新密码
changepwd-confirm-password-label = 确认新密码
changepwd-confirm-password-placeholder = 再次输入新密码
changepwd-submit = 修改密码

## add_bookmark.html / edit_bookmark.html（共用的书签表单）

bookmark-add-heading = 添加书签
bookmark-edit-heading = 编辑书签
bookmark-title-placeholder = 标题
bookmark-url-placeholder = 链接
bookmark-cover-image-placeholder = 封面图片
bookmark-upload-button = 上传
bookmark-new-tags-placeholder = 新增标签（逗号分隔）
bookmark-desc-placeholder = 描述
bookmark-add-submit = 添加
bookmark-edit-submit = 修改

## 共用的图片上传弹窗（add_bookmark.html / edit_bookmark.html）

upload-modal-title = 上传封面图片
upload-modal-select-image-label = 选择图片
upload-modal-cancel = 取消
upload-modal-submit = 上传

# 由 JS 驱动的上传状态提示
upload-status-uploading = 上传中...
upload-status-success = 上传成功！
upload-status-fail = 上传失败：{ $error }
upload-status-error-unknown = 未知错误
upload-status-error-network = 上传失败：网络错误

## home.html

home-mobile-menu = 菜单
home-search-placeholder = 搜索书签...
home-category-section-title = 分类
home-category-all = 全部
home-tag-section-title = 标签
home-language-section-title = 语言

# 用户菜单（侧边栏，仅管理员可见的部分项）
usermenu-add-bookmark = 添加书签
usermenu-import-bookmark = 导入书签
usermenu-change-password = 修改密码
usermenu-logout = 退出

# User Api
invalid_permission = 无效权限
invalid_credentials = 用户名或密码错误
username_required = 用户名不能为空
password_required = 密码不能为空
current_password_required = 当前密码不能为空
new_password_required = 新密码不能为空
confirm_password_required = 请确认新密码
password_mismatch = 两次输入的新密码不一致
current_password_wrong = 当前密码错误


title_required = 标题不能为空
url_required = URL不能为空
cover_image_required = 封面图片不能为空
category_required = 请选择分类

add_success = 添加成功
add_failed = 添加失败，请稍后重试
update_failed = 修改失败，请稍后重试

token_heading = 令牌生成
token_submit = 生成令牌
token_success = 令牌生成成功
copy_text = 复制
