

# def generate_add_applies(add_arg, ty, type_generic_args):
#     content = "apply Add<" + add_arg + "> to " + ty
#     if type_generic_args == "":
#         pass
#     else:
#         content += "<" + type_generic_args + ">"
#     content += """{
#     type Output = u32;
#     fn add(other u32) -> This::Output => this + other
#     fn add_unchecked(other u32) -> This::Output => this + other
# }"""
#     """apply Add<> to u32 {
#     type Output = u32;
#     fn add(other u32) -> This::Output => this + other
#     fn add_unchecked(other u32) -> This::Output => this + other
# }"""

if __name__ == "__main__":
    content = """
//- bench.flx

pub trait Add<T> {
    type Output;
    fn add(other T) -> This::Output;
    fn add_unchecked(other T) -> This::Output;
}
"""

    for i in range(10000):
        content += """apply Add<u32> to u32 {
    type Output = u32;
    fn add(other u32) -> This::Output => this + other
    fn add_unchecked(other u32) -> This::Output => this + other
}
"""

    f = open("bench.flx", "w")
    f.write(content)
    f.close()
    
