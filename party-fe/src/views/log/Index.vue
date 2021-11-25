<template>
<div class="tea-page">
  <h4>Reward Logs (Total: {{my_total_reward}})</h4>
  <el-button size="small" class="tea-refresh-btn" type="primary" plain icon="el-icon-refresh" circle @click="refreshPage()"></el-button>

  <TeaTable
    :data="my_reward || []"
    name="reward_log_table"
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
      label="CML ID"
    >
      <template slot-scope="scope">
        {{scope.row.cmlId}}
      </template>
    </el-table-column>

    <el-table-column
      label="Reward"
    >
      <template slot-scope="scope">
        {{scope.row.price}}
      </template>
    </el-table-column>
    
    <el-table-column
      prop="atBlock"
      label="At block"
      width="100"
    >
      <template slot-scope="scope">
        {{scope.row.atBlock}}
      </template>
    </el-table-column>
  </TeaTable>
  

  <el-divider />

  <h4>Operation Logs</h4>
  <TeaTable
    :data="my_log || []"
    name="operation_log_table"
    :pagination="true"
  >
    <el-table-column
      prop="type"
      label="Type"
      width="120"
    >
      <template slot-scope="scope">
        {{scope.row.type === 'tx' ? 'transaction': scope.row.type}}
      </template>
    </el-table-column>
    <el-table-column
      prop="name"
      label="Name"
    >
      <template slot-scope="scope">
        {{scope.row.name}}
      </template>
    </el-table-column>

    <el-table-column
      label="Auction ID"
    >
      <template slot-scope="scope">
        <el-button
          @click="showAuctionDetails(scope.row.auctionId)"
          type="text"
          size="small">
          {{scope.row.auctionId}}
        </el-button>
      </template>
    </el-table-column>

    <el-table-column
      label="CML ID"
    >
      <template slot-scope="scope">
        {{scope.row.cmlId}}
      </template>
    </el-table-column>

    <el-table-column
      label="Price"
    >
      <template slot-scope="scope">
        {{scope.row.price}}
      </template>
    </el-table-column>
    
    <el-table-column
      label="Amount"
    >
      <template slot-scope="scope">
        {{scope.row.amount}}
      </template>
    </el-table-column>

    <el-table-column
      label="Target user"
    >
      <template slot-scope="scope">
        {{scope.row.target}}
      </template>
    </el-table-column>

    <!-- <el-table-column
      prop="args"
      label="Arguments"
    >
      <template slot-scope="scope">
        {{scope.row.args}}
      </template>
    </el-table-column> -->
    <el-table-column
      prop="atBlock"
      label="At block"
      width="100"
    >
      <template slot-scope="scope">
        {{scope.row.atBlock}}
      </template>
    </el-table-column>

    <el-table-column
      label="Actions"
      width="120">
      <template slot-scope="scope">
        <el-link class="tea-action-icon" title="Bids" :underline="false" type="primary" icon="el-icon-view" @click="viewLogDetails(scope)"></el-link>
        
      </template>
    </el-table-column>
  </TeaTable>

  

</div>
</template>

<script>
import {_} from 'tearust_utils';
import utils from '../../tea/utils';
import { mapState } from 'vuex';
import store from '../../store/index';
import TeaTable from '../../components/TeaTable';
export default {
  components: {
    TeaTable,
  },
  data(){
    return {
      
    };
  },
  computed: {
    ...mapState('clog', {
      my_log:state => store.state.clog.my_log,
      my_reward:state => store.state.clog.my_reward,
      my_total_reward:state => store.state.clog.my_total_reward,
    })
  },
  async mounted(){
    this.$root.loading(true);
    await this.refreshPage();
    this.$root.loading(false);
  },
  methods: {
    async refreshPage(){
      this.$root.loading(true);
      await this.$store.dispatch('clog/init_my_auction_log', {});
      await this.$store.dispatch('clog/fetch_my_reward_log', {});
      this.$root.loading(false);
    },

    showAuctionDetails(auction_id){
      this.$store.commit('modal/open', {
        key: 'log_details', 
        param: {
          type: 'auction_id', 
          value: auction_id,
        },
      });
    },
    async viewLogDetails(scope){
      const param = {};

      let arr = null;
      try{
        let str = scope.row.args.replace(/\{/g, '[').replace(/\}/g, ']');
        arr = JSON.parse(str);
      }catch(e){
        arr = [];
      }

      _.each(arr, (val, i)=>{
        _.set(param, i+1, val);
      });
      param.title = 'Log details';
      this.$store.commit('modal/open', {
        key: 'data_details',
        param,
      });
      
    }
  }
  
}
</script>