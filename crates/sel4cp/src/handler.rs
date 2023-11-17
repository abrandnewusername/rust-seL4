use crate::cspace::{Channel, INPUT_CAP, REPLY_CAP, SIGNAL_QUEUED};
use crate::is_passive;
use crate::message::MessageInfo;

pub trait Handler {
    type Error;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        panic!("unexpected notification from channel {channel:?}")
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        panic!("unexpected protected procedure call from channel {channel:?} with msg_info={msg_info:?}")
    }

    fn run(&mut self) -> Result<!, Self::Error> {
        assert!(!is_passive());
        let mut reply_tag: Option<MessageInfo> = None;
        loop {
            // let mut badge: Badge = 0;
            // if let Some(tag) = reply_tag {
            //     tag, badge = INPUT_CAP.reply_recv(tag.into_sel4(), REPLY_CAP);
            // } else if let Some((send_cap, msg)) = SIGNAL_QUEUED {
            //     tag, badge = INPUT_CAP.nb_send_recv(send_cap, msg, REPLY_CAP);
            // } else {
            //     tag, badge = INPUT_CAP.recv(REPLY_CAP);
            // }

            let (tag, badge) = match reply_tag {
                Some(tag) => INPUT_CAP.reply_recv(tag.into_sel4(), REPLY_CAP),
                None => {
                    unsafe {
                        if let Some((send_cap, _)) = &SIGNAL_QUEUED {
                            INPUT_CAP.nb_send_recv(MessageInfo::new(0, 0).into_sel4(), *send_cap, REPLY_CAP)
                        } else {
                            INPUT_CAP.recv(REPLY_CAP)
                        }
                    }
                }
            };

            unsafe {
                SIGNAL_QUEUED = None;
            }
            // let (tag, badge) = match reply_tag {
            //     Some(tag) => INPUT_CAP.reply_recv(tag.into_sel4(), REPLY_CAP),
            //     None => {
            //         if Some()
            //         INPUT_CAP.recv(REPLY_CAP),
            //     }
            // };

            let tag = MessageInfo::from_sel4(tag);

            let is_endpoint = badge & (1 << (sel4::WORD_SIZE - 1)) != 0;

            reply_tag = if is_endpoint {
                let channel_index = badge & (sel4::Word::try_from(sel4::WORD_SIZE).unwrap() - 1);
                Some(self.protected(Channel::new(channel_index.try_into().unwrap()), tag)?)
            } else {
                let mut badge_bits = badge;
                while badge_bits != 0 {
                    let i = badge_bits.trailing_zeros();
                    self.notified(Channel::new(i.try_into().unwrap()))?;
                    badge_bits &= !(1 << i);
                }
                None
            };
        }
    }
}

pub struct NullHandler(());

impl NullHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

impl Handler for NullHandler {
    type Error = !;
}
