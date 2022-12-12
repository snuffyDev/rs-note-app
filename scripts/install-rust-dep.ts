#!/usr/bin/env node

import Sade from 'sade'
import {spawnSync} from 'child_process'
import { resolve } from 'path';

const cli = Sade('install-rust-dep');

cli.command('install <pkg>', "installs a rust crate").option("-f --features", 'installs the package with features').action((name, args, opts) => {
    spawnSync("cargo", args['features'] ? ['add', name, '-F', args['features']] : ['add', name], {cwd: resolve(process.cwd(), 'src-tauri'), stdio: 'inherit',
    shell: true});

    process.stdout.write(`\nFinished running cargo add ${name}\n`);
})

cli.parse(process.argv)