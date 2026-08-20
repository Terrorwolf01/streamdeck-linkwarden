<script lang="ts">
	import {onMount} from "svelte";
	import {actionSettings} from "@openaction/svelte-pi";
	import {refreshConnection} from "$lib/connection";
	import ConnectionSettings from "$lib/ConnectionSettings.svelte";

	let editingButton = false;
	let showOverrides = false;

    let tokenOverride = "";
    let instanceUrlOverride = "";

    $: {
        if ($actionSettings.token != undefined) {
            tokenOverride = $actionSettings.token;
        }
        if ($actionSettings.instanceUrl != undefined) {
            instanceUrlOverride = $actionSettings.instanceUrl;
        }
    }

    onMount(() => {
        // Start expanded if an override is already set, so an existing override isn't hidden
        // behind the checkbox with no indication it's there.
        if (tokenOverride || instanceUrlOverride) {
            showOverrides = true;
        }
    });

    function handleSaveButton() {
        $actionSettings = {
            ...$actionSettings,
            token: tokenOverride,
            instanceUrl: instanceUrlOverride,
        };
        editingButton = false;
        // The override may have just changed which credentials are actually in effect for this
        // instance, so the indicator in <ConnectionSettings /> needs to re-check against them.
        refreshConnection();
    }

    function handleCancelButton() {
        tokenOverride = $actionSettings.token || "";
        instanceUrlOverride = $actionSettings.instanceUrl || "";
        editingButton = false;
    }

    function maskSecret(secret: string): string {
        return secret ? "•".repeat(secret.length) : "";
    }
</script>

<ConnectionSettings />

<p class="mb-4 text-xs text-neutral-400">Opens the instance URL above in your browser when pressed.</p>

<hr class="mb-4 border-neutral-700" />

<h2 class="mb-3 text-sm font-semibold text-neutral-100">
    Button Settings
</h2>

<div class="mb-2 flex items-center gap-2">
    <input id="showOverrides" type="checkbox" bind:checked={showOverrides} class="cursor-pointer" />
    <label for="showOverrides" class="text-xs font-medium text-neutral-200"> Show override settings </label>
</div>

{#if showOverrides}
    <div class="mb-2 flex items-center gap-2">
        <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Override Token: </span>
        {#if editingButton}
            <input
                    id="tokenOverride"
                    type="password"
                    bind:value={tokenOverride}
                    placeholder="Optional, overrides the global token for this button"
                    class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
            />
        {:else}
            <span class="text-xs text-neutral-400">{maskSecret(tokenOverride) || "Not set"}</span>
        {/if}
    </div>
    <p class="mb-2 text-xs text-neutral-500">
        Opening Linkwarden doesn't require a token - this is only used for the connection check above.
    </p>

    <div class="mb-2 flex items-center gap-2">
        <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Override URL: </span>
        {#if editingButton}
            <input
                    id="instanceUrlOverride"
                    type="text"
                    bind:value={instanceUrlOverride}
                    placeholder="Optional, overrides the global instance URL for this button"
                    class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
            />
        {:else}
            <span class="text-xs text-neutral-400">{instanceUrlOverride || "Not set"}</span>
        {/if}
    </div>

    {#if editingButton}
        <div class="mb-3 flex gap-2">
            <button
                    on:click={handleSaveButton}
                    class="cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
            >
                Save
            </button>
            <button
                    on:click={handleCancelButton}
                    class="cursor-pointer rounded bg-neutral-800 px-3 py-1 text-xs text-neutral-300 hover:bg-neutral-700"
            >
                Cancel
            </button>
        </div>
    {:else}
        <button
                on:click={() => (editingButton = true)}
                class="cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
        >
            Edit
        </button>
    {/if}
{/if}