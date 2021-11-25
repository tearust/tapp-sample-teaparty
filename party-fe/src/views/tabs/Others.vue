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
      prop="type"
      label="Type"
    />

    <el-table-column
      prop="from"
      label="From"
    />

    <el-table-column
      label="Buy price"
    >
      <template slot-scope="scope">
        <span>{{showJSON(scope, 'buy_price')}}</span>
      </template>
    </el-table-column>
    <!-- <el-table-column
      label="Sell price"
    >
      <template slot-scope="scope">
        <span>{{showJSON(scope, 'sell_price')}}</span>
      </template>
    </el-table-column> -->

    <el-table-column
      label="TEA amount"
    >
      <template slot-scope="scope">
        <span>{{showJSON(scope, 'tea_amount')}}</span>
      </template>
    </el-table-column>
    <el-table-column
      label="Token amount"
    >
      <template slot-scope="scope">
        <span>{{showJSON(scope, 'token_amount')}}</span>
      </template>
    </el-table-column>

    

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

    utils.register('home__other', async (key, param)=>{
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
        type: `equalTo: "event"`,
        name: `notIn: ["TAppExpense", "TAppConsume", "TAppConsumeRewardStatements", "updateTappLastActivity"]`,
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

    showJSON(scope, key){
      let json = scope.row.json;
      if(!json) return '';

      json = JSON.parse(json);

      return _.get(json, key, '');
    }


  }
};

</script>
