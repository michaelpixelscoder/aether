import {defineConfig} from '@playwright/test';
export default defineConfig({
  testDir:'./e2e', timeout:120000, expect:{timeout:20000}, workers:1,
  outputDir:'../docs/evidence/browser-artifacts',
  reporter:[['list'],['json',{outputFile:'../docs/evidence/browser-tests.json'}]],
  use:{channel:process.env.AETHER_BROWSER||'msedge', baseURL:process.env.AETHER_TEST_URL||'http://127.0.0.1:4173', viewport:{width:1280,height:720},
    launchOptions:{args:['--use-angle=d3d11','--enable-unsafe-webgpu']}, screenshot:'only-on-failure',trace:'retain-on-failure'}
});
