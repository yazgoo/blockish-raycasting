pub struct Level {
    pub url: String,
    pub world_map: Vec<Vec<u8>>,
    pub world_layer: Vec<Vec<u8>>,
    pub sprites: Vec<Vec<f32>>,
    pub portals: Vec<Vec<f32>>,
    pub portals_destinations: Vec<Vec<f32>>,
    pub on_action: Box<dyn Fn(f32, f32, u8, &mut Vec<Vec<u8>>, &mut Vec<Vec<u8>>) -> ()>,
}

pub fn _first() -> Level {
    Level {
        url: String::from("https://srv-file10.gofile.io/download/GrF7ZN/wolfenstein_textures.zip"),
        world_map: vec![
        vec![8,8,8,8,8,8,8,8,8,8,8,4,4,6,4,4,6,4,6,4,4,4,6,4],
        vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
        vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,6],
        vec![8,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6],
        vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
        vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,6,6,6,0,6,4,6],
        vec![8,8,8,8,0,8,8,8,8,8,8,4,4,4,4,4,4,6,0,0,0,0,0,6],
        vec![7,7,7,7,0,7,7,7,7,0,8,0,8,0,8,0,8,4,0,4,0,6,0,6],
        vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,0,0,0,0,0,6],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,0,0,0,0,4],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,6,0,6,0,6],
        vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,4,6,0,6,6,6],
        vec![7,7,7,7,0,7,7,7,7,8,8,4,0,6,8,4,8,3,3,3,0,3,3,3],
        vec![2,2,2,2,0,2,2,2,2,4,6,4,0,0,6,0,6,3,0,0,0,0,0,3],
        vec![2,2,0,0,0,0,0,2,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
        vec![2,0,0,0,0,0,0,0,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
        vec![1,0,0,0,0,0,0,0,1,4,4,4,4,4,6,0,6,3,3,0,0,0,3,3],
        vec![2,0,0,0,0,0,0,0,2,2,2,1,2,2,2,6,6,0,0,5,0,5,5,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,5,5,0,5,0,0,0,5,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,5,5,5,5,0,0,0,5,5],
        vec![2,2,2,2,1,2,2,2,2,2,2,1,2,2,2,5,5,5,5,5,5,5,5,5]
            ],
            world_layer: vec![
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
                    ],
                    sprites: vec![
                        //green light in front of playerstart
                        vec![20.5, 11.5, 10.0],
                        //green lights in every room
                        vec![18.5,4.5, 10.0],
                        vec![10.0,4.5, 10.0],
                        vec![10.0,12.5,10.0],
                        vec![3.5, 6.5, 10.0],
                        vec![3.5, 20.5,10.0],
                        vec![3.5, 14.5,10.0],
                        vec![14.5,20.5,10.0],

                        //row of pillars in front of wall: fisheye test
                        vec![18.5, 10.5, 9.0],
                        vec![18.5, 11.5, 9.0],
                        vec![18.5, 12.5, 9.0],

                        //some barrels around the map
                        vec![21.5, 1.5, 8.0],
                        vec![15.5, 1.5, 8.0],
                        vec![16.0, 1.8, 8.0],
                        vec![16.2, 1.2, 8.0],
                        vec![3.5,  2.5, 8.0],
                        vec![9.5, 15.5, 8.0],
                        vec![10.0, 15.1,8.0],
                        vec![10.5, 15.8,8.0],
                        ],
                        portals: vec![
                            vec![20.5, 10.1, 0.0],
                            vec![10.0, 10.0, 1.0],
                        ],
                        portals_destinations: vec![
                            vec![10.0, 10.0],
                            vec![20.5, 10.1],
                        ],
                        on_action: Box::new(|_x, _y, _action, _world_map, _level| {  }),
    }
}
pub fn _rat_race() -> Level {
    Level { url: String::from("https://srv-file10.gofile.io/download/GrF7ZN/wolfenstein_textures.zip"),
    world_map: vec![
        vec![4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,0,4,],
        vec![4,0,4,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,4,0,4,],
        vec![4,0,4,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,4,0,4,],
        vec![4,0,4,7,0,7,7,7,7,7,7,0,7,7,7,7,7,7,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,6,6,6,6,0,6,6,6,6,6,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,0,0,0,0,0,0,0,0,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,6,6,6,6,6,6,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,2,2,1,2,2,2,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,2,0,0,0,0,2,6,0,6,7,0,7,4,0,4,],
        vec![4,0,0,0,0,7,6,0,0,0,0,0,0,0,0,0,0,6,7,0,0,0,0,4,],
        vec![4,0,4,7,0,7,6,0,6,2,0,2,2,0,2,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,2,0,0,0,0,2,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,2,2,1,2,2,2,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,6,6,6,6,6,6,6,6,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,0,0,0,0,0,0,0,0,0,0,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,6,6,6,6,6,6,0,6,6,6,6,6,7,0,7,4,0,4,],
        vec![4,0,4,7,0,7,7,7,7,7,7,7,0,7,7,7,7,7,7,0,7,4,0,4,],
        vec![4,0,4,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,4,0,4,],
        vec![4,0,4,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,4,0,4,],
        vec![4,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,],
        ],
        world_layer: vec![
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
                ],
                sprites: vec![
                    vec![13.75,10.291666666666666,9.0],
                    vec![10.208333333333332,10.291666666666666,9.0],
                    vec![1.375,1.4166666666666665,8.0],
                    vec![22.75,1.5,8.0],
                    vec![1.4166666666666665,20.625,8.0],
                    vec![4.416666666666667,19.375,8.0],
                    vec![1.5416666666666665,11.416666666666666,10.0],
                    vec![22.458333333333332,11.375,10.0],
                    vec![12.0,11.625,10.0],
                    vec![7.541666666666666,11.5,10.0],
                    vec![16.5,11.458333333333334,10.0],
                    vec![12.458333333333332,18.125,10.0],
                    vec![11.375,5.875,10.0],
                    vec![22.708333333333336,20.166666666666668,8.0],
                    vec![7.666666666666666,1.0833333333333333,9.0],
                    vec![17.0,1.1666666666666667,9.0],
                    vec![7.583333333333333,4.208333333333333,9.0],
                    vec![16.666666666666664,4.333333333333333,9.0],
                ],
                portals: vec![],
                portals_destinations: vec![],
                on_action: Box::new(|_x, _y, _action, _world_map, _level| {  }),
    }
}
pub fn _spyral() -> Level {
    Level { url: String::from("https://srv-file10.gofile.io/download/GrF7ZN/wolfenstein_textures.zip"),
    world_map: vec![
        vec![2,2,1,2,2,2,2,2,1,2,2,2,2,2,2,1,2,2,2,2,2,2,2,2,],
        vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,],
        vec![2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,0,1,],
        vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,2,],
        vec![2,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,0,2,0,2,],
        vec![2,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,2,0,2,],
        vec![1,0,2,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,0,0,0,0,0,0,0,0,0,0,0,4,0,2,0,2,0,1,],
        vec![2,0,2,0,4,0,4,4,4,4,4,4,4,4,4,4,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,0,0,0,0,0,0,0,0,4,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,0,5,5,5,5,5,5,0,4,0,4,0,2,0,2,0,2,],
        vec![1,0,2,0,4,0,4,0,5,0,0,0,0,5,0,4,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,0,5,0,0,6,0,5,0,4,0,4,0,2,0,2,0,1,],
        vec![2,0,2,0,4,0,4,0,5,0,5,5,5,5,0,4,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,0,5,0,0,0,0,0,0,4,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,0,5,5,5,5,5,5,5,4,0,4,0,2,0,2,0,2,],
        vec![1,0,2,0,4,0,4,0,0,0,0,0,0,0,0,0,0,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,4,4,4,4,4,4,4,4,4,4,4,4,0,2,0,2,0,2,],
        vec![2,0,2,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,2,0,1,],
        vec![2,0,2,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,2,0,2,0,2,],
        vec![2,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,2,],
        vec![2,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,0,2,],
        vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,],
        vec![2,1,2,2,2,2,2,1,2,2,2,2,2,1,2,2,2,2,2,1,2,2,2,2,],
        ],
        world_layer: vec![
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
                ],
                sprites: vec![
                    vec![5.375,1.5416666666666665,10.0],
                    vec![12.791666666666668,1.5,10.0],
                    vec![22.458333333333332,6.958333333333334,10.0],
                    vec![22.458333333333332,16.083333333333332,10.0],
                    vec![1.4583333333333333,14.166666666666668,10.0],
                    vec![3.458333333333333,17.208333333333332,10.0],
                    vec![3.4166666666666665,11.166666666666668,10.0],
                    vec![5.583333333333334,10.708333333333334,10.0],
                    vec![7.458333333333334,13.916666666666668,10.0],
                    vec![16.375,11.875,10.0],
                    vec![18.625,9.458333333333332,10.0],
                    vec![18.708333333333332,15.5,10.0],
                    vec![20.416666666666664,12.0,10.0],
                    vec![1.4166666666666665,3.3333333333333335,9.0],
                    vec![20.708333333333332,3.291666666666667,9.0],
                    vec![9.291666666666668,14.625,9.0],
                    vec![9.25,11.416666666666666,9.0],
                    vec![11.541666666666668,11.5,8.0],
                    vec![12.416666666666668,12.583333333333334,8.0],
                    vec![16.291666666666664,16.5,8.0],
                    vec![5.416666666666667,18.583333333333336,8.0],
                    vec![3.458333333333333,20.5,8.0],
                    vec![3.666666666666667,5.5,8.0],
                    vec![10.333333333333334,3.208333333333333,8.0],
                    vec![20.583333333333332,20.166666666666668,8.0],
                    vec![22.708333333333336,1.3333333333333333,8.0],
                    ],
                    portals: vec![
                        vec![1.5, 1.5, 0.0],
                        vec![11.5, 11.5, 1.0],
                    ],
                    portals_destinations: vec![
                        vec![11.5, 11.5],
                        vec![1.5, 10.1],
                    ],
                    on_action: Box::new(|_x, _y, _action, _world_map, _level| {  }),
    }
}
pub fn _trapped() -> Level {
    Level { url: String::from("https://srv-file10.gofile.io/download/GrF7ZN/wolfenstein_textures.zip"),
    world_map: vec![
        vec![7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,7,7,7,7,7,7,7,7,7,7,7,7,0,7,7,7,7,7,7,7,7,7,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,],
        vec![7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,],

        ],
        world_layer: vec![
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
                ],
                sprites: vec![
                    vec![17.958333333333332,17.833333333333336,10.0],
                    vec![17.875,5.75,10.0],
                    vec![6.416666666666666,5.791666666666667,10.0],
                    vec![6.375,18.458333333333332,10.0],
                    vec![1.3333333333333333,1.3333333333333333,8.0],
                    vec![11.708333333333332,1.2916666666666667,8.0],
                    vec![13.458333333333332,2.2083333333333335,8.0],
                    vec![13.875,1.5833333333333335,8.0],
                    vec![22.583333333333332,9.5,8.0],
                    vec![22.166666666666668,17.375,8.0],
                    vec![13.416666666666668,13.583333333333332,8.0],
                    vec![11.541666666666668,19.291666666666664,8.0],
                    vec![11.625,18.833333333333332,8.0],
                    vec![1.2916666666666667,13.416666666666668,9.0],
                    vec![1.25,11.541666666666668,9.0],
                    vec![13.333333333333334,11.5,9.0],
                    vec![22.625,11.416666666666666,9.0],
                    vec![22.458333333333332,13.583333333333332,9.0],
                    vec![13.208333333333332,18.333333333333332,9.0],

                    ],
                    portals: vec![
                        vec![1.5, 1.5, 0.0],
                        vec![20.5, 20.5, 1.0],
                    ],
                    portals_destinations: vec![
                        vec![20.5, 20.1],
                        vec![1.5, 1.5],
                    ],
                    on_action: Box::new(|_x, _y, _action, _world_map, _level| {  }),
    }
}
pub fn metro() -> Level {
    Level { url: String::from("metro/metro.zip"),
    world_map: vec![
        vec![4,4,4,4,4,4,4,4,4,4,4,4,4,4,16,4,4,4,4,4,4,4,4,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,],
        vec![4,0,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,13,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6,6,6,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,],
        vec![4,0,12,1,8,1,2,1,1,2,8,1,3,1,8,3,1,2,1,8,1,3,1,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,10,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6,6,6,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,11,11,0,11,11,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,5,5,0,11,11,0,11,0,0,0,0,0,13,0,0,0,0,16,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,11,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,11,11,12,11,11,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,13,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,0,0,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,],
        vec![4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,16,4,4,4,4,4,4,4,4,],


        ],
        world_layer: vec![
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,17],
            vec![18,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,19,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,18],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,18,0,17,0,0,0,0,0],
            vec![17,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![18,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,20,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            vec![0,0,0,0,0,0,0,0,0,17,0,0,0,0,0,0,0,0,0,0,0,0,17,0]
                ],
                sprites: vec![
                    vec![17.125,5.875,13.0],
                    vec![7.375,8.333333333333332,13.0],
                    vec![3.125,18.291666666666668,13.0],
                    vec![16.833333333333332,20.708333333333332,13.0],
                    vec![17.833333333333332,20.708333333333332,14.0],
                    vec![4.125,12.291666666666668,14.0],

                ],
                portals: vec![
                    vec![1.5, 1.5, 0.0],
                    vec![20.5, 20.5, 1.0],
                ],
                portals_destinations: vec![
                    vec![20.5, 20.1],
                    vec![1.5, 1.5],
                ],
                on_action: Box::new(|x, y, _action, world_map, world_layer| { 
                    if x as usize == 21 && y as usize == 3 {
                        if world_layer[22][3] == 20 {
                            world_layer[22][3] = 21;
                            world_map[22][4] = 0;
                        } else {
                            world_layer[22][3] = 20;
                            world_map[22][4] = 4;
                        }
                    }
                }),
    }
}
