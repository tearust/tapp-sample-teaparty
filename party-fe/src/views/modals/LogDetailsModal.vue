<template>
  <el-dialog
    :title="(param && param.title) || 'Log details'"
    :visible="visible"
    width="90%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @close="$store.commit('modal/close', 'log_details')"
    @open="refreshList()"
  >

    <el-table 
      :data="details || []"
      stripe
      size="small"
      border
    >
      <el-table-column
        prop="type"
        label="Type"
        width="80"
      >
        <template slot-scope="scope">
          {{scope.row.type === 'tx' ? 'transaction': scope.row.type}}
        </template>
      </el-table-column>
      <el-table-column
        prop="name"
        label="Name"
        width="140"
      >
        <template slot-scope="scope">
          {{scope.row.name}}
        </template>
      </el-table-column>

      <el-table-column
        label="Auction Id"
        width="80"
      >
        <template slot-scope="scope">
          {{scope.row.auctionId}}
        </template>
      </el-table-column>

      <el-table-column
        label="CML Id"
        width="60"
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
        label="From User"
      >
        <template slot-scope="scope">
          {{scope.row.from}}
        </template>
      </el-table-column>

      <el-table-column
        label="To User"
      >
        <template slot-scope="scope">
          {{scope.row.target}}
        </template>
      </el-table-column>

      <el-table-column
        prop="atBlock"
        label="At Block"
        width="100"
      >
        <template slot-scope="scope">
          {{scope.row.atBlock}}
        </template>
      </el-table-column>
      
    </el-table>

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="$store.commit('modal/close', 'log_details')">Close</el-button>
    </span>

  </el-dialog>


</template>
<script>
import { mapState } from 'vuex';
import store from '../../store/index';
import utils from '../../tea/utils';
export default {
  data(){
    return {

    };
  },
  computed: {
    ...mapState('modal', {
      visible: state => store.state.modal.log_details.visible,
      param: state => store.state.modal.log_details.param,
    }),
    ...mapState('clog', {
      details: state => store.state.clog.details,
    }),
  },

  methods: {
    async refreshList(){
      if(!this.param || !this.param.type) return;
      const opts = {
        type: this.param.type,
        value: this.param.value,
      };
      this.$root.loading(true);
      await this.$store.dispatch('clog/fetch_details_log', opts);
      this.$root.loading(false);
    }
  }
}
</script>