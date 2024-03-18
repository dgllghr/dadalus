mod maze;
mod wilsons;

use wilsons::Generator;

fn main() {
    let size = (100, 100);

    let mut rng = rand::thread_rng();

    let generator = Generator::new(size.0, size.1);
    let maze = generator.generate(&mut rng);

    let pixmap = maze.draw(25);
    pixmap.save_png("image.png").unwrap();
}
