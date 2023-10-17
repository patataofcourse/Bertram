pub struct FaqQuestion {
    pub name: &'static str,
    pub question: &'static str,
    pub answer: &'static str,
}

impl FaqQuestion {
    const fn new(name: &'static str, q: &'static str, a: &'static str) -> Self {
        Self {
            name,
            question: q,
            answer: a,
        }
    }
}

pub const RESOURCES_TAG: &str = concat!(
        "Before asking any questions, please look through the game's section in <#832643992528355368>. ",
        "If you're trying to mod or play mods for Megamix, you're probably looking for the ",
        "[RHM Beginner's Guide](https://docs.google.com/document/d/1FvCB0bL-Zt17wuOThXy-P8lObFSMogl_kftN87PyLaI/edit)."
);

pub const QUESTIONS: &[FaqQuestion] = &[
    //TODO: link to spicerack
    FaqQuestion::new(
        "intl",
        "How can I get mods running on my Japanese/European/Korean copy of Megamix?",
        concat!(
            "It's not that simple, sadly. RHMPatch *only* supports US copies of Megamix, and ", 
            "a small amount of mods would be incompatible anyway. For EU/KR, you can try out [SpiceRack], ",
            "although it's still in beta and experimental, and doesn't support as wide of an array of mods ", 
            "as RHMPatch. For JP, it's more complicated than that, and while support is eventually planned, ",
            "it's not guaranteed."
        )
    ),
    //TODO: zlib merging guide?
    FaqQuestion::new(
        "multimod",
        "Can I run multiple mods at the same time? How?",
        concat!(
            "The answer depends on whether you're using RHMPatch or SpiceRack\n\n",

            "**For RHMPatch:** Yes, but it's not without its set of complications. You'll need to set up Tickompiler, ",
            "merge your tickflow .bin files into one C00, and merge matching files* (usually at least" ,
            "`USmessage/pajama.zlib` and `USlayout/coffee_game.zlib`). For the first two, you can follow the \"How to ",
            "Combine Multiple Cue Mods\" section in the [Beginner's Guide](https://docs.google.com/document/d/1FvCB0bL-Zt17wuOThXy-P8lObFSMogl_kftN87PyLaI). ",
            "Merging .zlib files is more complicated, and you can follow XYZ guide (AN: does it exist rn?)\n\n",

            "**For SpiceRack:** Yes! SpiceRack is intended to eventually lead to seamless mod merging, however, this ",
            "only applies to Tickflow right now. You will still need to merge files such as `pajama.zlib`\\* (see above).\n\n",

            "\\* Most .zlib files don't need to be merged, however, not merging them will result in missing museum assets, broken text, and potential crashes.",
        )
    ),
    FaqQuestion::new(
        "recs",
        "Why do recreations like Rap Men / Manzai / etc. exist, but Toss Boys / Polyrhythm / etc. don't?",
        concat!(
            "The answer to that question is very complex, however, it can usually be pinpointed to the requirements for ",
            "that recreation being too strict or otherwise unachievable with current modding tech. To see what the ",
            "recreating process looks like, we recommend reading over Piyo Piyori and Terraria Tree's [Porting Guide](https://docs.google.com/document/d/1GH3ee-ZsrkhblP22VDa2mZVjCROHB2gZIXavIVHl74o/edit),",
            "although it is somewhat outdated."
        )
    ),
    FaqQuestion::new(
        "startup",
        "I installed RHMPatch and Megamix crashes before starting up!",
        concat!(
            "You might be missing the C00.bin file in your SD:/rhmm folder. Even if you don't want to run any mods ",
            "at the time, you need a C00.bin. You can use the empty C00.bin included in RHMPatch's zip file, or ",
            "rename a base.bin to C00.bin."
        )
    ),
    FaqQuestion::new(
        "emu",
        "Can I run Megamix mods on an emulator?",
        concat!(
            "Yes! You can run both RHMPatch mods and SpiceRack mods on Citra (although not at the same time). See ",
            "\"Getting Started (Citra)\" on the [Beginner's Guide](https://docs.google.com/document/d/1FvCB0bL-Zt17wuOThXy-P8lObFSMogl_kftN87PyLaI/edit) ",
            "for RHMPatch, and the [SpiceRack quick setup guide for Citra](https://patataofcourse.github.io/spicerack-guide/citra.html) for Spicerack."
        )
    ),
    FaqQuestion::new(
        "emucompat",
        "Can I run Megamix on an emulator?",
        concat!(
            "Compatibility for Megamix in 3DS emulators is as follows:\n",
            "- Citra (PC): Runs mostly fine. Audio might be slightly desynced and emulation of crashes/errors is not accurate.\n",
            "- Citra (Mobile): Similar to PC Citra, but will perform worse and have worse input lag.\n",
            "- Panda3DS: Doesn't go farther than a couple menus. Can't run mods."
        )
    ),
    FaqQuestion::new(
        "loaders",
        "What's RHMPatch? What's SpiceRack?",
        concat!(
            "RHMPatch is the current stable loader to run Megamix mods, which only works on US Megamix. SpiceRack is an experimental ",
            "mod loader which works on US/EU/KR Megamix and allows loading multiple Tickflow mods at once, however, it's missing some ",
            "features of RHMPatch and is overall less polished."
        )
    )
];
