async function main() {
    try {
        const response = await fetch('./rustycheckers.wasm');
        const bytes = await response.arrayBuffer();
        const results = await WebAssembly.instantiate(bytes, {
            env: {
                notify_piece_moved: (fx, fy, tx, ty) => {
                    console.log("A piece moved from (" + fx + "," + fy + ") to (" + 
                        tx + "," + ty + ")");
                },

                notify_piece_crowned: (x, y) => {
                    console.log("A piece was crowned at (" + x + "," + y + ")");
                }
            },
        });

        const instance = results.instance;
        
        console.log("At start, current turn is " + instance.exports.get_current_turn());
        const piece = instance.exports.get_piece(0, 7);
        console.log("Piece at 0,7 is: " + piece);

        console.log(instance.exports);

        let res = instance.exports.move_piece(0, 5, 1, 4);
        console.log("First move result: " + res);
        console.log("Turn after move: " + instance.exports.get_current_turn());

        let bad = instance.exports.move_piece(1, 4, 2, 3);
        console.log("Illegal move result: " + bad);
        console.log("Turn after illegal move: " + instance.exports.get_current_turn());
    } catch(err) {
        console.error(err);
    }
}

main();
