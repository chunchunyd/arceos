# 查找根目录下所有cargo.toml文件
# 用法：python localize.py dep_name
# 作用：将所有依赖dep_name的项目的依赖路径修改为本地路径, 需要提前将.cargo中的依赖库移动到extern_crates目录下

import os
import re
import sys

# 项目根目录
root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
# print(root_dir)

# 遍历所有文件，查找cargo.toml文件
cargo_toml_list = []
for root, dirs, files in os.walk(root_dir):
    for file in files:
        if file == 'Cargo.toml':
            cargo_toml_list.append(os.path.join(root, file))
# print(cargo_toml_list)

# 对于一个依赖项dep_name，其依赖项的版本号在Cargo.toml文件中的格式为：
# 1. dep_name = "version"
# 2. dep_name = { version = "version", ... }
# 3. dep_name = { path = "path", ... }     (我们的目的是转换为这种情况)
# 4. [dependencies.dep_name]\nversion = "version"
# 4. [dependencies.dep_name]\npath = "path" (我们的目的是转换为这种情况)

op = sys.argv[1]    # localize or restore
dep_name = sys.argv[2]
if op == 'restore':
    version = sys.argv[3]

print(dep_name)
# 在root_dir下的extern_crates目录下，找到前缀为dep_name的目录
dep_local_path = ''
has_dep = False
for root, dirs, files in os.walk(os.path.join(root_dir, 'extern_crates')):
    for dir in dirs:
        if dir.startswith(f'{dep_name}-'):
            if has_dep:
                print('Error: More than one dependency found!')
                print('Please check the extern_crates directory.')
                exit(1)
            dep_local_path = os.path.join(root, dir)
            has_dep = True
print(dep_local_path)

# 查找所有包含依赖项dep_name的Cargo.toml文件，并将其依赖项的版本号转换为依赖于本地路径
# cargo_toml = '/home/ccyd/HW/RustOS/ArceOS/arceos/scripts/Cargo.toml'
# cargo_toml = '/home/ccyd/HW/RustOS/ArceOS/arceos/extern_crates/bitflags-2.1.0/Cargo.toml'
for cargo_toml in cargo_toml_list:
    # 计算dep_local_path相对于cargo_toml的路径
    dep_local_path_rel = os.path.relpath(dep_local_path, os.path.dirname(cargo_toml))
    with open(cargo_toml, 'r', encoding='utf-8') as f:
        content = f.read()
        # 1. dep_name = "version"
        pattern = re.compile(rf'\n{dep_name}\s*=\s*"\d+(\.((\d+)|\*))*"')        # \s
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        if op == 'localize':
            content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}" }}', content)
        elif op == 'restore':
            content = re.sub(pattern, f'\n{dep_name} = {{ version = "{version}" }}', content)
        # 2. dep_name = { version = "version", ... }
        pattern = re.compile(rf'\n{dep_name}\s*=\s*{{\s*version\s*=\s*"\d+(\.\d+)*"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        if op == 'localize':
            content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}"', content)
        elif op == 'restore':
            content = re.sub(pattern, f'\n{dep_name} = {{ version = "{version}"', content)
        # 3. dep_name = { path = "path", ... }
        pattern = re.compile(rf'\n{dep_name}\s*=\s*{{\s*path\s*=\s*"(.+?)"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        if op == 'localize':
            content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}"', content)
        elif op == 'restore':
            content = re.sub(pattern, f'\n{dep_name} = {{ version = "{version}"', content)
        # 4. [dependencies.dep_name]\nversion = "version"
        pattern = re.compile(rf'\.{dep_name}\]\s*version\s*=\s*"\d+(\.((\d+)|\*))*"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        if op == 'localize':
            content = re.sub(pattern, f'.{dep_name}]\npath = "{dep_local_path_rel}"', content)
        elif op == 'restore':
            content = re.sub(pattern, f'.{dep_name}]\nversion = "{version}"', content)
        # 5. [dependencies.dep_name]\npath = "path"
        pattern = re.compile(rf'\.{dep_name}\]\s*path\s*=\s*"(.+?)"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        if op == 'localize':
            content = re.sub(pattern, f'.{dep_name}]\npath = "{dep_local_path_rel}"', content)
        elif op == 'restore':
            content = re.sub(pattern, f'.{dep_name}]\nversion = "{version}"', content)

        with open(cargo_toml, 'w', encoding='utf-8') as f:
            f.write(content)
