<script lang="ts">
  export let title: string;
</script>

<div class="container">
  <div class="title">{title}</div>
  <div class="content">
    <slot />
  </div>
</div>

<style>
  .container {
    border: 1px solid #ccc;
    border-radius: 2px;
    overflow: hidden;
  }
  .title {
    text-align: center;
    background-color: #ddd;
    color: #666;
    font-size: 0.9em;
    font-weight: bold;
    user-select: none;
  }
  .content {
    padding: var(--panel-contnet-padding, 10px);
  }
</style>
