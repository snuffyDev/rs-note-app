<script context="module" lang="ts">
	import { writable } from "svelte/store";
	const file = writable<{
		uuid: string;
		content: { file_name: string; content: string };
	}>({
		uuid: "",
		content: { content: "", file_name: "" },
	});
</script>

<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import Greet from "./lib/Greet.svelte";
	import { marked } from "marked";

	let files = [];
	let output = "";
	$: output = marked($file.content.content, {
		gfm: true,
		smartypants: true,
		smartLists: true,
	});
	const onClickSave = async () => {
		let uuid = $file.uuid || crypto.randomUUID();
		const data = (await invoke("save_file", {
			fileName: $file.content.file_name,
			uuid,
			content: JSON.stringify($file.content),
		})) as any[];
		files = [...data];
		console.log(files, data);
	};
	const onClick = async () => {
		const data = (await invoke("get_files")) as any[];
		files = [...data];
		console.log(data);
	};
	const onItemClick = (file) => {};
	onClick();
</script>

<main class="container">
	<input placeholder="Title" type="text" bind:value={$file.content.file_name} />
	<div class="split">
		<div class="col">
			<textarea bind:value={$file.content.content} cols="30" rows="10" />
		</div>
		<div class="col">{@html output}</div>
	</div>

	<button on:click={onClickSave}>Save</button>
	<button on:click={onClick}>Load all notes</button>

	<div class="row">
		<Greet />
	</div>
	<ul>
		{#each files as f}
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<li
				on:click={() =>
					file.update((u) => ({ ...f, content: JSON.parse(f.content) }))}
			>
				<div class="col">
					<p>{f?.uuid}</p>
					<p>{f?.content}</p>
				</div>
			</li>
		{/each}
	</ul>
</main>

<style lang="scss">
	.split {
		display: grid;
		grid-template-columns: 1fr 1fr;
	}
	li {
		margin: 0.25em 0;
		padding: 0.25em 1em;
		list-style: none;
		user-select: none !important;
		-webkit-user-select: none;
		cursor: pointer;
		&:hover {
			background-color: rgba(164, 161, 161, 0.326);
		}
	}
	ul {
		margin: 0;
		padding: 0;
	}
	.col {
		display: flex;
		flex-direction: column;
		text-align: unset;
	}
	.col p {
		margin: 0.15em 0;
	}
	.logo.vite:hover {
		filter: drop-shadow(0 0 2em #747bff);
	}

	.logo.svelte:hover {
		filter: drop-shadow(0 0 2em #ff3e00);
	}
</style>
