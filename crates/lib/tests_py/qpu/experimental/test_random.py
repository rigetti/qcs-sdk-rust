from qcs_sdk.qpu.experimental.random import ChooseRandomRealSubRegions


def test_choose_random_real_subregions():
    """Test the name and signature used to declare `ChooseRandomRealSubRegions` in a Quil program."""
    assert ChooseRandomRealSubRegions.NAME == "choose_random_real_sub_regions"
    assert (
        ChooseRandomRealSubRegions.build_signature()
        == "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
    )
