fn export_db() {
    let pg_dump = cmd([
        "docker",
        "exec",
        "-e",
        "PGPASSWORD=db",
        "ddev-beeraffordability-db",
        "pg_dump",
        "-U",
        "db",
        "-Ft",
        "-f",
        "/tmp/postgres-dump.tar",
    ]);
    pg_dump.execute();

    let cp = cmd([
        "docker",
        "cp",
        "ddev-beeraffordability-db:/tmp/postgres-dump.tar",
        "./postgres-dump.tar",
    ]);
    cp.execute();
}

fn publish() {
    let registry = "registry.s2.goodserver.ch/beeraffordability";

    publish_craft(registry);
	publish_svelte(registry);
}

fn publish_craft(registry) {
    // make sure all volumes are owned by 3000
    let build = cmd([
        "docker",
        "buildx",
        "build",
        "--platform",
        "linux/amd64",
        "--push",
        "-t",
        registry + "/craft",
        ".",
    ]);
    build.dir("craft");
    build.execute();
}

fn publish_svelte(registry) {
    // make sure all volumes are owned by 1000
    let build = cmd([
        "docker",
        "buildx",
        "build",
        "--platform",
        "linux/amd64",
        "--push",
        "-t",
        registry + "/svelte",
        ".",
    ]);
    build.dir("svelte");
    build.execute();
}
