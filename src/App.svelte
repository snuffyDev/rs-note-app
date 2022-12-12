<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import Greet from './lib/Greet.svelte'
let file_name = '';
let content = ''
  const onClickSave = async() => {
    const data = await invoke('save_file', { file_name: file_name, content});
    console.log(data);
  }
  const onClick = async() => {
    const data = await invoke('get_files');
    console.log(data);
  }
</script>

<main class="container">

  <input placeholder="Title" type="text" bind:value={file_name}>
  <textarea bind:value={content} cols="30" rows="10"></textarea>

  <button on:click={onClickSave}>Save</button>
  <button on:click={onClick}>Load all notes</button>

  <div class="row">
    <Greet />
  </div>


</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }
</style>