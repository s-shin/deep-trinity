<script lang="ts">
  import { type Statistics, StatisticsEntryType } from "@deep-trinity/model";
  export let stats: Statistics;
  export let types: StatisticsEntryType[] = [
    StatisticsEntryType.Single,
    StatisticsEntryType.Double,
    StatisticsEntryType.Triple,
    StatisticsEntryType.Tetris,
    StatisticsEntryType.Tst,
    StatisticsEntryType.Tsd,
    StatisticsEntryType.Tss,
    StatisticsEntryType.Tsmd,
    StatisticsEntryType.Tsms,
    StatisticsEntryType.MaxCombos,
    StatisticsEntryType.MaxBtbs,
    StatisticsEntryType.PerfectClear,
    StatisticsEntryType.Hold,
    StatisticsEntryType.Lock,
  ];
  export let labels = {
    [StatisticsEntryType.Single]: "SINGLE",
    [StatisticsEntryType.Double]: "DOUBLE",
    [StatisticsEntryType.Triple]: "TRIPLE",
    [StatisticsEntryType.Tetris]: "TETRIS",
    [StatisticsEntryType.Tst]: "TST",
    [StatisticsEntryType.Tsd]: "TSD",
    [StatisticsEntryType.Tss]: "TSS",
    [StatisticsEntryType.Tsmd]: "TSMD",
    [StatisticsEntryType.Tsms]: "TSMS",
    [StatisticsEntryType.MaxCombos]: "MAX COMBO",
    [StatisticsEntryType.MaxBtbs]: "MAX BTB",
    [StatisticsEntryType.PerfectClear]: "PC",
    [StatisticsEntryType.Hold]: "HOLD",
    [StatisticsEntryType.Lock]: "LOCK",
  };
</script>

<table>
  {#each types as t}
    <tr>
      <th>{labels[t]}</th>
      <td>{stats[t]}</td>
    </tr>
  {/each}
</table>

<style>
  table {
    font-size: 0.9em;
    border: 1px solid #ccc;
    border-radius: 2px;
    border-collapse: collapse;
    background-color: #ddd;
  }
  th {
    color: #666;
    padding: 0 0.5em;
    border-bottom: 1px solid #fff;
  }
  tr:last-child th {
    border-bottom: 1px solid #ccc;
  }
  td {
    color: #333;
    min-width: 5em;
    padding: 0 0.5em;
    border-bottom: 1px solid #ccc;
    box-sizing: border-box;
    background-color: #fff;
  }
</style>
