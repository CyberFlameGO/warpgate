<script lang="ts">
import { api } from 'lib/api'
import { authenticatedUsername } from 'lib/store'
import { replace } from 'svelte-spa-router'

import { Alert, Button, FormGroup } from 'sveltestrap'
let error: Error|null = null
let username = ''
let password = ''
let incorrectCredentials = false

function onInputKey (event: KeyboardEvent) {
    if (event.key === 'Enter') {
        login()
    }
}

async function login () {
    error = null
    incorrectCredentials = false
    try {
        await api.login({
            loginRequest: {
                username,
                password,
            },
        })
    } catch (error) {
        if (error.status === 401) {
            incorrectCredentials = true
        } else {
            error = error
        }
        return
    }
    const info = await api.getInfo()
    authenticatedUsername.set(info.username!)
    replace('/')
}
</script>

<div class="mt-5 row">
    <div class="col-12 col-md-3"></div>
    <div class="col-12 col-md-6">
        <div class="page-summary-bar">
            <h1>Welcome</h1>
        </div>

        <FormGroup floating label="Username">
            <!-- svelte-ignore a11y-autofocus -->
            <input
                bind:value={username}
                on:keypress={onInputKey}
                class="form-control"
                autofocus />
        </FormGroup>

        <FormGroup floating label="Password">
            <input
                bind:value={password}
                on:keypress={onInputKey}
                type="password"
                class="form-control" />
        </FormGroup>

        <Button
            outline
            on:click={login}
        >Login</Button>

        {#if incorrectCredentials}
            <Alert color="danger">Incorrect credentials</Alert>
        {/if}
        {#if error}
            <Alert color="danger">{error}</Alert>
        {/if}

    </div>
    <div class="col-12 col-md-3"></div>
</div>
