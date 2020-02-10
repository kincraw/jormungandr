use crate::blockcfg::{
    BlockDate, ChainLength, Header, HeaderHash, Leadership, Ledger, LedgerParameters,
};
use chain_impl_mockchain::multiverse;
use chain_time::{
    era::{EpochPosition, EpochSlotOffset},
    Epoch, Slot, TimeFrame,
};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

/// a reference to a block in the blockchain
#[derive(Clone)]
pub struct Ref {
    /// Reference holder for the object in the `Multiverse<Ledger>`.
    ledger: multiverse::Ref<Ledger>,

    /// the time frame applicable in the current branch of the blockchain
    time_frame: Arc<TimeFrame>,

    /// the leadership used to validate the current header's leader
    ///
    /// this object will be shared between different Ref of the same epoch
    epoch_leadership_schedule: Arc<Leadership>,

    /// pointer to the current ledger parameters
    ///
    /// The object will be shared between different Ref of the same epoch
    epoch_ledger_parameters: Arc<LedgerParameters>,

    /// keep the Block header in memory, this will avoid retrieving
    /// the data from the storage if needs be
    header: Header,

    /// holder to the previous epoch state or more precisely the previous epoch's
    /// last `Ref`. Every time there is a transition this value will be filled with
    /// the parent `Ref`. Otherwise it will be copied from `Ref` to `Ref`.
    ///
    previous_epoch_state: Option<Arc<Ref>>,
}

impl Ref {
    /// create a new `Ref`
    pub fn new(
        ledger: multiverse::Ref<Ledger>,
        time_frame: Arc<TimeFrame>,
        epoch_leadership_schedule: Arc<Leadership>,
        epoch_ledger_parameters: Arc<LedgerParameters>,
        header: Header,
        previous_epoch_state: Option<Arc<Ref>>,
    ) -> Self {
        debug_assert_eq!(
            *ledger.id(),
            header.hash(),
            "expect the ledger reference to be for the same `Header`"
        );

        Ref {
            ledger,
            time_frame,
            epoch_leadership_schedule,
            epoch_ledger_parameters,
            header,
            previous_epoch_state,
        }
    }

    /// retrieve the header hash of the `Ref`
    pub fn hash(&self) -> HeaderHash {
        *self.ledger.id()
    }

    /// access the reference's parent hash
    pub fn block_parent_hash(&self) -> HeaderHash {
        self.header().block_parent_hash()
    }

    /// retrieve the block date of the `Ref`
    pub fn block_date(&self) -> BlockDate {
        self.header().block_date()
    }

    /// retrieve the chain length, the number of blocks created
    /// between the block0 and this block. This is useful to compare
    /// the density of 2 branches.
    pub fn chain_length(&self) -> ChainLength {
        self.header().chain_length()
    }

    /// access the `Header` of the block pointed by this `Ref`
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn ledger(&self) -> Arc<Ledger> {
        self.ledger.state_arc()
    }

    /// get the time frame in application in the current branch of the blockchain
    pub fn time_frame(&self) -> &Arc<TimeFrame> {
        &self.time_frame
    }

    pub fn epoch_leadership_schedule(&self) -> &Arc<Leadership> {
        &self.epoch_leadership_schedule
    }

    pub fn epoch_ledger_parameters(&self) -> &Arc<LedgerParameters> {
        &self.epoch_ledger_parameters
    }

    pub fn last_ref_previous_epoch(&self) -> Option<&Arc<Ref>> {
        self.previous_epoch_state.as_ref()
    }

    /// get the chain_time's `Slot`. This allows to compute an accurate
    /// block time via a given time_frame or a precise block time
    pub fn slot(&self) -> Slot {
        let era = self.epoch_leadership_schedule().era();

        let epoch = Epoch(self.header.block_date().epoch);
        let slot = EpochSlotOffset(self.header.block_date().slot_id);

        era.from_era_to_slot(EpochPosition { epoch, slot })
    }

    /// retrieve the time of the associated block.
    pub fn time(&self) -> SystemTime {
        let slot = self.slot();
        let time_frame = self.time_frame();

        if let Some(time) = time_frame.slot_to_systemtime(slot) {
            time
        } else {
            // this case cannot happen because we cannot have a time_frame
            // change during the lifetime of the object.

            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    /// retrieve the time of the slot of the block. If the block is set
    /// in the future, this function will return an error.
    pub fn elapsed(&self) -> Result<Duration, std::time::SystemTimeError> {
        SystemTime::now().duration_since(self.time())
    }
}
