use crate::tools::glv_env_control;

pub fn wgpu_setup_main(password: &str) -> std::io::Result<()> {
    let wsl_cmds = vec![ // it is essential pkg installs for wsl bevy or etcs that need only display showing. :) 
        "cd /", 
        "sudo apt install -y build-essential cmake pkg-config libudev-dev libssl-dev libx11-dev libxi-dev libgl1-mesa-dev libglu1-mesa-dev mesa-common-dev libxrandr-dev libxxf86vm-dev libasound2-dev vulkan-tools mesa-vulkan-drivers",
        "echo 'export WGPU_BACKEND=vulkan' >> ~/.bashrc",
        "source ~/.bashrc"
    ];

    glv_env_control("wsl", wsl_cmds, password);

    Ok(())
}
