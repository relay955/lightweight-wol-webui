<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {goto} from "$app/navigation";
  import {onMount} from "svelte";

  let user_name = '';
  let password = '';
  let isLoading = false;
  let isChecking = true;

  // 페이지 로드 시 계정 존재 여부 확인
  onMount(showToastOnError(async () => {
    const res = await axios.get('/check-first-user');
    if (!res.data.has_user) {
      // 계정이 없으면 초기 설정 페이지로 리다이렉트
      goto('/setup');
    } else {
      isChecking = false;
    }
  }));

  const onClickLogin = showToastOnError(async () => {
    isLoading = true;
    await axios.post('/login', {user_name, password});
    goto('/main');
  }, () => isLoading = false);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') onClickLogin();
  }
</script>

<div class="auth-container">
  <div class="auth-card">
    {#if isChecking}
      <div class="checking-container">
        <span class="spinner large"></span>
        <p class="checking-text">Checking system status...</p>
      </div>
    {:else}
      <h1 class="page-title">Lightweight WOL WebUI</h1>
      <p class="page-subtitle">Authentication</p>

      <form class="common-form" on:submit|preventDefault={onClickLogin}>
        <div class="form-group">
          <label for="account">Username</label>
          <input 
            id="account" 
            type="text" 
            on:keydown={onKeyDown}
            placeholder="Enter your username" 
            bind:value={user_name}
            disabled={isLoading}
          />
        </div>

        <div class="form-group">
          <label for="password">Password</label>
          <input 
            id="password" 
            type="password" 
            on:keydown={onKeyDown}
            placeholder="Enter your password" 
            bind:value={password}
            disabled={isLoading}
          />
        </div>

        <button class="action-button" type="button" on:click={onClickLogin} disabled={isLoading}>
          {#if isLoading}
            <span class="spinner"></span>
            Logging in...
          {:else}
            Login
          {/if}
        </button>
      </form>
    {/if}
  </div>
</div>

<style lang="scss">
  /* 페이지별 특화 스타일이 필요한 경우 여기에 추가 */
</style>
