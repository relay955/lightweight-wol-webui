<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {onMount} from "svelte";
  import {toast} from "@zerodevx/svelte-toast";

  interface Device {
    id: number;
    name: string;
    mac: string;
    order_num: number;
  }

  let devices: Device[] = $state([]);
  let isLoading = $state(true);

  // 장치 목록 불러오기
  const loadDevices = showToastOnError(async () => {
    isLoading = true;
    const res = await axios.get('/devices');
    devices = res.data.devices;
    isLoading = false;
  }, () => isLoading = false);

  // 장치 삭제
  const deleteDevice = (id: number) => showToastOnError(async () => {
    if (!confirm('이 장치를 삭제하시겠습니까?')) return;
    await axios.delete(`/devices/${id}`);
    toast.push('장치가 삭제되었습니다.', {
      theme: {
        '--toastBackground': '#1a1a1a',
        '--toastColor': '#ffffff',
        '--toastBarBackground': '#4caf50'
      }
    });
    await loadDevices();
  })();

  onMount(() => {
    loadDevices();
  });
</script>

<div class="main-container">
  <div class="main-content">
    <!-- 헤더 섹션 -->
    <div class="header-section">
      <div class="title-area">
        <h1 class="main-title">WOL Devices</h1>
        <p class="main-subtitle">Wake-on-LAN 장치 관리</p>
      </div>
      <button class="add-button action-button">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        장치 추가
      </button>
    </div>

    <!-- 로딩 상태 -->
    {#if isLoading}
      <div class="checking-container">
        <span class="spinner large"></span>
        <p class="checking-text">장치 목록을 불러오는 중...</p>
      </div>
    {:else if devices.length === 0}
      <!-- 빈 상태 -->
      <div class="empty-state">
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
          <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
        </svg>
        <h3>등록된 장치가 없습니다</h3>
        <p>새로운 WOL 장치를 추가해보세요.</p>
      </div>
    {:else}
      <!-- 장치 리스트 -->
      <div class="devices-grid">
        {#each devices as device (device.id)}
          <div class="device-card">
            <div class="device-info">
              <div class="device-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
                  <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
                </svg>
              </div>
              <div class="device-details">
                <h3 class="device-name">{device.name}</h3>
                <p class="device-mac">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect>
                    <line x1="12" y1="18" x2="12.01" y2="18"></line>
                  </svg>
                  {device.mac}
                </p>
              </div>
            </div>
            <div class="device-actions">
              <button class="icon-button edit-button" title="수정">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </button>
              <button class="icon-button delete-button" title="삭제" onclick={() => deleteDevice(device.id)}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  <line x1="10" y1="11" x2="10" y2="17"></line>
                  <line x1="14" y1="11" x2="14" y2="17"></line>
                </svg>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  .main-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);
    padding: 40px 20px;
    position: relative;

    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-image: 
        repeating-linear-gradient(0deg,
            transparent, transparent 2px,
            rgba(0, 0, 0, 0.015) 2px,
            rgba(0, 0, 0, 0.015) 4px
        );
      pointer-events: none;
    }
  }

  .main-content {
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
  }

  /* 헤더 섹션 */
  .header-section {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 40px;
    flex-wrap: wrap;
    gap: 20px;
  }

  .title-area {
    flex: 1;
  }

  .main-title {
    font-size: 36px;
    font-weight: bold;
    color: var(--color-black);
    margin: 0 0 8px 0;
    letter-spacing: -0.5px;
  }

  .main-subtitle {
    font-size: 16px;
    color: var(--color-dark-gray);
    margin: 0;
    font-weight: normal;
  }

  .add-button {
    padding: 12px 24px;
    font-size: 15px;
    white-space: nowrap;
  }

  /* 장치 그리드 */
  .devices-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 24px;
    animation: fadeIn 0.4s ease-in-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* 장치 카드 */
  .device-card {
    background: var(--color-white);
    border-radius: 12px;
    padding: 24px;
    border: 2px solid var(--color-border);
    transition: var(--transition-base);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);

    &:hover {
      border-color: var(--color-black);
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
      transform: translateY(-2px);
    }
  }

  .device-info {
    display: flex;
    align-items: center;
    gap: 16px;
    flex: 1;
    min-width: 0;
  }

  .device-icon {
    width: 48px;
    height: 48px;
    background: var(--color-light-gray);
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border: 2px solid var(--color-border);
    transition: var(--transition-base);

    svg {
      color: var(--color-black);
    }

    .device-card:hover & {
      background: var(--color-black);
      border-color: var(--color-black);

      svg {
        color: var(--color-white);
      }
    }
  }

  .device-details {
    flex: 1;
    min-width: 0;
  }

  .device-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-black);
    margin: 0 0 6px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .device-mac {
    font-size: 13px;
    color: var(--color-dark-gray);
    margin: 0;
    font-family: 'Courier New', monospace;
    display: flex;
    align-items: center;
    gap: 6px;

    svg {
      flex-shrink: 0;
    }
  }

  /* 액션 버튼들 */
  .device-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .icon-button {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: 2px solid var(--color-border);
    background: var(--color-light-gray);
    cursor: pointer;
    transition: var(--transition-base);
    display: flex;
    align-items: center;
    justify-content: center;

    svg {
      color: var(--color-black);
      transition: var(--transition-base);
    }

    &:hover {
      transform: translateY(-2px);
    }

    &.edit-button:hover {
      background: var(--color-black);
      border-color: var(--color-black);

      svg {
        color: var(--color-white);
      }
    }

    &.delete-button:hover {
      background: #dc3545;
      border-color: #dc3545;

      svg {
        color: var(--color-white);
      }
    }
  }

  /* 빈 상태 */
  .empty-state {
    text-align: center;
    padding: 80px 20px;

    svg {
      color: var(--color-dark-gray);
      opacity: 0.4;
      margin-bottom: 24px;
    }

    h3 {
      font-size: 22px;
      font-weight: 600;
      color: var(--color-black);
      margin: 0 0 12px 0;
    }

    p {
      font-size: 15px;
      color: var(--color-dark-gray);
      margin: 0;
    }
  }

  /* 반응형 디자인 */
  @media (max-width: 768px) {
    .main-container {
      padding: 24px 16px;
    }

    .header-section {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;
    }

    .title-area {
      text-align: center;
    }

    .main-title {
      font-size: 28px;
    }

    .main-subtitle {
      font-size: 14px;
    }

    .add-button {
      width: 100%;
    }

    .devices-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .device-card {
      padding: 20px;
    }
  }

  @media (max-width: 480px) {
    .device-card {
      flex-direction: column;
      align-items: stretch;
    }

    .device-info {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .device-actions {
      justify-content: center;
      width: 100%;
      gap: 12px;

      .icon-button {
        flex: 1;
        max-width: 120px;
      }
    }
  }
</style>