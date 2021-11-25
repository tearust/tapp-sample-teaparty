<template>
<div class="tea-table-box">
<el-table 
  v-bind="{...$props, ...$attrs}" 
  v-on="$listeners" 
  size="small"
  :ref="name"
  :data="list||[]"
  stripe
  border
  @sort-change="sortChangeHandler"
  class="tea-table">
  <slot></slot>
</el-table>
<el-pagination
  v-if="pagination"
  hide-on-single-page
  background
  style="text-align: right; margin: 10px -10px 40px;"
  :current-page.sync="current"
  @current-change="changePage($event)"
  @prev-click="changePage($event)"
  @next-click="changePage($event)"
  :page-size.sync="size"
  layout="prev, pager, next"
  :total="total" />
</div>

</template>
<script>
import {_} from 'tearust_utils';
import utils from '../tea/utils';
export default {
  inheritAttrs: false,
  data(){
    return {
      all_list: null,
      list: null,
      current: 1,
      total: 0,
    };
  },
  props: {
    name: {
      type: String,
      required: true,
    },
    size: {
      type: Number,
      default: 10
    },
    pagination: {
      type: Boolean,
      default: false,
    }
  },
  mounted(){    
    const key = this.sort_key();
    const default_sort = utils.mem.get(key);
    const ref = this.$refs[this.name];
    if(default_sort && ref && ref.sort){
      ref.sort(default_sort.prop, default_sort.order);
    }

    this.refresh();

  },
  watch: {
    $attrs() {
      this.refresh();
    }
  },
  methods: {
    refresh(){
      this.all_list = this.$attrs.data;
      if(this.pagination){
        this.total = this.all_list.length;
        this.changePage(1);
      }
      else{
        this.list = this.all_list;
      }
    },
    sort_key(){
      return this.name+'__sort';
    },
    sortChangeHandler({order, prop}){
      const key = this.sort_key();
      utils.mem.set(key, {
        order, prop
      });
    },
    changePage(val){
      this.current = val;
      const from = (this.current-1)*this.size;
      const to = this.current*this.size;
      this.list = _.slice(this.all_list, from, to);
    }
  }
}
</script>