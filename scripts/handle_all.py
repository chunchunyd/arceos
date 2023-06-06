import os


# 项目根目录
root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))

extern_crates_path = os.path.join(root_dir, 'extern_crates')
extern_crates = os.listdir(extern_crates_path)
extern_crates = [crate for crate in extern_crates if os.path.isdir(os.path.join(extern_crates_path, crate))]
# print(extern_crates)

dep_names = []
for crate in extern_crates:
    dep_names.append(crate.split('-')[0])
