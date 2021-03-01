pub fn main() -> u32 {
    mut u32 x = 0;

    /*
     * '&&' has greater precedence than '||'
     * so this is (x<10 or x==10) && x == 1
    */
    if(x < 10 || x == 10 && x == 0) {
        x = 20;
    }

    return x;
}