# 重命名一个 crate 引用
# 用法: python rename_crate.py <old_crate_name> <new_crate_name> dir_path
# 作用: 例如extern_crates目录下有两个crate: syn-1.0.109 和 syn-2.0.13
#      但是同一个工作目录下中只能有一个syn，因此需要将其中一个重命名
#      例如将syn-1.0.109重命名为syn_1
#      此时依赖syn-1.0.109的项目中的use syn;语句需要修改为use syn_1; use syn_1::xxx;需要修改为use syn_1::xxx;
#      这个脚本就是用来修改这些语句的，暂时使用还比较麻烦，需要手动指定每一个引用syn-1.0.109的项目的路径

import os
import re
import sys

# 项目根目录
root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
# print(root_dir)

# 要修改的 crate 名称
old_crate_name = sys.argv[1]
new_crate_name = sys.argv[2]
print(f'old_crate_name: {old_crate_name}')
print(f'new_crate_name: {new_crate_name}')

# 依赖这个 crate 的 项目路径
dir_path = sys.argv[3]
print(f'dir_path: {dir_path}')

# 遍历dir_path下所有.rs文件，将old_crate_name替换为new_crate_name；将cargo.toml中的依赖名称修改为new_crate_name
for root, dirs, files in os.walk(dir_path):
    for file in files:
        if file.endswith('.rs'):
            with open(os.path.join(root, file), 'r', encoding='utf-8') as f:
                content = f.read()
                # 1. use old_crate_name
                pattern = re.compile(rf'use\s+{old_crate_name}\s*;')
                content = re.sub(pattern, f'use {new_crate_name};', content)
                # 2. use old_crate_name::xxx
                pattern = re.compile(rf'use\s+{old_crate_name}::')
                content = re.sub(pattern, f'use {new_crate_name}::', content)
                # 3. old_crate_name::xxx
                pattern = re.compile(rf'{old_crate_name}::')
                content = re.sub(pattern, f'{new_crate_name}::', content)
                with open(os.path.join(root, file), 'w', encoding='utf-8') as f:
                    f.write(content)
        elif file == 'Cargo.toml':
            with open(os.path.join(root, file), 'r', encoding='utf-8') as f:
                content = f.read()
                # 4. old_crate_name = "xxx"
                pattern = re.compile(rf'{old_crate_name}\s*=\s*"\S+"')
                content = re.sub(pattern, f'{new_crate_name} = "{{ path = \\"../{old_crate_name}\\" }}"', content)
                # 5. old_crate_name = { version = "xxx" }
                pattern = re.compile(rf'{old_crate_name}\s*=\s*{{\s*version\s*=\s*"\S+"\s*}}')
                content = re.sub(pattern, f'{new_crate_name} = {{ path = "../{old_crate_name}" }}', content)
                # 6. dependencies.old_crate_name
                pattern = re.compile(rf'dependencies\.{old_crate_name}')
                content = re.sub(pattern, f'dependencies.{new_crate_name}', content)

                with open(os.path.join(root, file), 'w', encoding='utf-8') as f:
                    f.write(content)
