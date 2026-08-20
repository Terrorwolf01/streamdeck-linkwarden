<script lang="ts">
	import {onMount} from "svelte";
	import {actionSettings} from "@openaction/svelte-pi";
	import {tags, collections, tagsError, collectionsError, refreshConnection, type Tag} from "$lib/connection";
	import ConnectionSettings from "$lib/ConnectionSettings.svelte";

	let editingButton = false;
	let showOverrides = false;

    let tokenOverride = "";
    let instanceUrlOverride = "";
    let description = "";

    $: {
        if ($actionSettings.token != undefined) {
            tokenOverride = $actionSettings.token;
        }
        if ($actionSettings.instanceUrl != undefined) {
            instanceUrlOverride = $actionSettings.instanceUrl;
        }
        if ($actionSettings.description != undefined) {
            description = $actionSettings.description;
        }
    }

    // Unlike the text/secret fields above, tag/collection are selected from a closed list, so
    // there's nothing to "commit" - each selection is applied immediately, matching how
    // OpenActionPlugins/discord's selectchannel page writes guild/channel picks straight to
    // $actionSettings on change rather than gating them behind an edit/save toggle.
    $: selectedTagIds = new Set(($actionSettings.tags ?? []).map((tag: Tag) => tag.id));
    $: selectedCollectionId = $actionSettings.collectionId ?? "";

    function toggleTag(tag: Tag, event: Event) {
        const checked = (event.target as HTMLInputElement).checked;
        const current: Tag[] = $actionSettings.tags ?? [];
        const tags = checked ? [...current, tag] : current.filter((t) => t.id !== tag.id);
        $actionSettings = {
            ...$actionSettings,
            tags,
        };
    }

    function updateCollection(event: Event) {
        const value = (event.target as HTMLSelectElement).value;
        const collectionId = value === "" ? undefined : Number(value);
        $actionSettings = {
            ...$actionSettings,
            collectionId,
            collectionName: $collections.find((collection) => collection.id === collectionId)?.name ?? "",
        };
    }

    function updateUsePageTitle(event: Event) {
        $actionSettings = {
            ...$actionSettings,
            usePageTitle: (event.target as HTMLInputElement).checked,
        };
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
            description,
        };
        editingButton = false;
        // The override may have just changed which credentials are actually in effect for this
        // instance, so the indicator in <ConnectionSettings /> needs to re-check against them.
        refreshConnection();
    }

    function handleCancelButton() {
        tokenOverride = $actionSettings.token || "";
        instanceUrlOverride = $actionSettings.instanceUrl || "";
        description = $actionSettings.description || "";
        editingButton = false;
    }

    function maskSecret(secret: string): string {
        return secret ? "•".repeat(secret.length) : "";
    }
</script>

<ConnectionSettings />

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
{/if}

<div class="mb-2 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Description: </span>
    {#if editingButton}
        <textarea
                id="description"
                bind:value={description}
                placeholder="Optional, overrides the default description used for added links"
                rows="2"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        ></textarea>
    {:else}
        <span class="text-xs text-neutral-400">{description || "Not set"}</span>
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
            class="mb-3 cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
    >
        Edit
    </button>
{/if}

<div class="mb-2 flex items-start gap-2">
    <span class="min-w-22.5 pt-1 text-xs font-medium text-neutral-200"> Tags: </span>
    <div class="flex flex-1 flex-wrap gap-x-3 gap-y-1 rounded border border-neutral-700 bg-neutral-800 p-2">
        {#if $tags.length === 0}
            <span class="text-xs text-neutral-500">No tags available</span>
        {/if}
        {#each $tags as tag}
            <label class="flex cursor-pointer items-center gap-1 text-xs text-neutral-200">
                <input
                        type="checkbox"
                        checked={selectedTagIds.has(tag.id)}
                        on:change={(event) => toggleTag(tag, event)}
                        class="cursor-pointer"
                />
                {tag.name}
            </label>
        {/each}
    </div>
</div>

<div class="mb-3 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Collection: </span>
    <div class="select-wrapper flex-1">
        <select
                id="collection"
                value={selectedCollectionId}
                on:change={updateCollection}
                class="w-full rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        >
            <option value="">Default (Unorganized)</option>
            {#each $collections as collection}
                <option value={collection.id}>{collection.name}</option>
            {/each}
        </select>
    </div>
</div>

<div class="mb-3 flex items-center gap-2">
    <input
            id="usePageTitle"
            type="checkbox"
            checked={$actionSettings.usePageTitle ?? false}
            on:change={updateUsePageTitle}
            class="cursor-pointer"
    />
    <label for="usePageTitle" class="text-xs font-medium text-neutral-200">
        Use the website's title as the link name (instead of the URL)
    </label>
</div>
{#if $tagsError || $collectionsError}
    <p class="mb-3 text-xs text-red-400">
        Failed to load {$tagsError && $collectionsError ? "tags and collections" : $tagsError ? "tags" : "collections"}: {$tagsError ||
            $collectionsError}
    </p>
{/if}