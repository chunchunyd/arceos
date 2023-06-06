import os
import re
import sys

# 项目根目录
root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
print(root_dir)

# 遍历所有文件，查找cargo.toml文件
cargo_toml_list = []
for root, dirs, files in os.walk(root_dir):
    for file in files:
        if file == 'Cargo.toml':
            cargo_toml_list.append(os.path.join(root, file))
# print(cargo_toml_list)

dep_name = 'axlog'
dep_local_path = os.path.abspath(os.path.join(os.path.dirname(__file__), '..','modules','axlog'))

for cargo_toml in cargo_toml_list:
    dep_local_path_rel = os.path.relpath(dep_local_path, os.path.dirname(cargo_toml))
    with open(cargo_toml, 'r', encoding='utf-8') as f:
        content = f.read()
        # 1. dep_name = "version"
        pattern = re.compile(rf'\n{dep_name}\s*=\s*"\d+(\.((\d+)|\*))*"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}" }}', content)
        # 2. dep_name = { version = "version", ... }
        pattern = re.compile(rf'\n{dep_name}\s*=\s*{{\s*version\s*=\s*"\d+(\.\d+)*"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}"', content)
        # 3. dep_name = { path = "path", ... }
        pattern = re.compile(rf'\n{dep_name}\s*=\s*{{\s*path\s*=\s*"(.+?)"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        content = re.sub(pattern, f'\n{dep_name} = {{ path = "{dep_local_path_rel}"', content)
        # 4. [dependencies.dep_name]\nversion = "version"
        pattern = re.compile(rf'\.{dep_name}\]\s*version\s*=\s*"\d+(\.((\d+)|\*))*"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        content = re.sub(pattern, f'.{dep_name}]\npath = "{dep_local_path_rel}"', content)
        # 5. [dependencies.dep_name]\npath = "path"
        pattern = re.compile(rf'\.{dep_name}\]\s*path\s*=\s*"(.+?)"')
        if re.search(pattern, content):
            print(f'Found {dep_name} in {cargo_toml}!')
        content = re.sub(pattern, f'.{dep_name}]\npath = "{dep_local_path_rel}"', content)

        with open(cargo_toml, 'w', encoding='utf-8') as f:
            f.write(content)