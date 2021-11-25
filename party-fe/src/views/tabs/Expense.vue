<template>
<div>
  
  <TeaTable
    v-loading="table_loading"
    :data="list || []"
    name="expense_log_table"
    :pagination="true"
  >
    <el-table-column
      prop="name"
      label="Name"
    >
      <template slot-scope="scope">
        {{scope.row.name}}
      </template>
    </el-table-column>

    <el-table-column
      prop="from"
      label="From"
    />
    <el-table-column
      prop="to"
      label="To"
    />
    <el-table-column
      prop="cmlId"
      label="CML ID"
    />

    <!-- <el-table-column
      label="Amount (TEA)"
    >
      <template slot-scope="scope">
        <span :inner-html.prop="scope.row.price | balance"></span>
      </template>
    </el-table-column> -->

    <el-table-column
      prop="atBlock"
      label="At block"
    />


    <el-table-column
      label="Actions"
      width="120">
      <template slot-scope="scope">
        <el-link class="tea-action-icon" :underline="false" type="primary" icon="el-icon-view" @click="viewLogDetails(scope)"></el-link>
        
      </template>
    </el-table-column>


  </TeaTable>

</div>
</template>
<script>
import { mapGetters, mapState } from 'vuex';
import Base from '../../workflow/Base';
import {_} from 'tearust_utils';
import utils from '../../tea/utils';
import request from '../../request';
import TeaTable from '../../components/TeaTable';
import {stringToHex, hexToString,} from 'tearust_layer1';

export default {
  components: {TeaTable},
  props: {
    id: {
      type: Number,
      required: true,
    }
  },
  data(){
    return {
      table_loading: false,
      list: null,
    };
  },
  computed: {
    ...mapGetters([
      'layer1_account'
    ]),
  },
  async mounted(){
    await this.refreshList();

    utils.register('home__expense', async (key, param)=>{
      await this.refreshList();   
    });
  },
  methods: {
    loading(flag=false){
      this.table_loading = flag;
      this.$root.loading(flag);
    },
    async refreshList(){
      this.loading(true);
      const list = await request.getLog({
        name: `in: ["TAppExpense"]`,
        tappId: `in: ["${this.id}"]`,
      });

      this.list = list.nodes;
      
      this.loading(false);
    },
    
    async viewLogDetails(scope){
      let param = {};
      const json = JSON.parse(scope.row.json);
      param = json;


      param.title = 'Log details';
      this.$store.commit('modal/open', {
        key: 'data_details',
        param,
      });
    },

    showNote(scope){
      if(scope.row.json){
        const json = JSON.parse(scope.row.json);
        if(json.note){
          return hexToString(json.note);
        }
        
      }
      return '';
    },


  }
};

</script>
